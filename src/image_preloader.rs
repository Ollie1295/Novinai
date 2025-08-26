use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::{mpsc, Mutex, Semaphore};
use reqwest::Client;
use bytes::Bytes;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use moka::future::Cache;
use dashmap::DashMap;
use url::Url;

#[derive(Debug, Clone)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug)]
pub struct ImageDownloadRequest {
    pub url: String,
    pub event_id: Uuid,
    pub priority: Priority,
    pub callback: Option<tokio::sync::oneshot::Sender<Result<Bytes, ImageError>>>,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum ImageError {
    #[error("http status {status}")]
    HttpStatus { status: u16 },
    #[error("network error: {0}")]
    Network(String),
    #[error("not found in cache")]
    NotFound,
    #[error("request cancelled")]
    Cancelled,
    #[error("download timeout")]
    Timeout,
    #[error("too large: {0} bytes")]
    TooLarge(usize),
    #[error("unsupported content-type: {0}")]
    UnsupportedContentType(String),
    #[error("invalid image format")]
    InvalidFormat,
}

pub struct ImagePreloader {
    cache: Cache<String, CacheEntry>,
    q_crit: mpsc::Sender<ImageDownloadRequest>,
    q_high: mpsc::Sender<ImageDownloadRequest>,
    q_norm: mpsc::Sender<ImageDownloadRequest>,
    q_low: mpsc::Sender<ImageDownloadRequest>,
    inflight: Arc<Mutex<HashMap<String, Vec<tokio::sync::oneshot::Sender<Result<Bytes, ImageError>>>>>>,
    per_host: Arc<DashMap<String, Arc<Semaphore>>>,
    client: Client,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: Bytes,
    timestamp: chrono::DateTime<chrono::Utc>,
    access_count: Arc<AtomicU32>,
}

// Constants for size limits and validation
const MAX_BYTES: usize = 5 * 1024 * 1024;   // 5MB cap
const RANGE_BYTES: usize = 2 * 1024 * 1024; // 2MB precheck

impl ImagePreloader {
    pub fn new() -> Self {
        // Create priority queues with backpressure
        let (q_crit_tx, mut q_crit_rx) = mpsc::channel::<ImageDownloadRequest>(128);
        let (q_high_tx, mut q_high_rx) = mpsc::channel::<ImageDownloadRequest>(256);
        let (q_norm_tx, mut q_norm_rx) = mpsc::channel::<ImageDownloadRequest>(512);
        let (q_low_tx, mut q_low_rx) = mpsc::channel::<ImageDownloadRequest>(512);

        // Create bounded cache with TTL and byte-based capacity
        let cache = Cache::builder()
            .max_capacity(100 * 1024 * 1024) // 100 MB total cached bytes
            .time_to_live(Duration::from_secs(3600)) // 1 hour TTL
            .weigher(|_k: &String, v: &CacheEntry| v.data.len() as u32) // weight = bytes
            .build();

        let inflight = Arc::new(Mutex::new(HashMap::new()));
        let per_host = Arc::new(DashMap::new());
        
        // Create optimized HTTP client
        let client = Client::builder()
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(10))
            .tcp_keepalive(Duration::from_secs(60))
            .user_agent("Novin/1.0")
            .build()
            .expect("Failed to create HTTP client");

        // Global concurrency cap
        let permits = Arc::new(Semaphore::new(32));

        // Priority-based worker loop
        let cache_c = cache.clone();
        let client_c = client.clone();
        let inflight_c = inflight.clone();
        let per_host_c = per_host.clone();
        let permits_c = permits.clone();
        tokio::spawn(async move {
            info!("Priority-based image preloader worker started");
            
            loop {
                let req = tokio::select! {
                    Some(r) = q_crit_rx.recv() => r,
                    Some(r) = q_high_rx.recv() => r,
                    Some(r) = q_norm_rx.recv() => r,
                    Some(r) = q_low_rx.recv() => r,
                    else => break,
                };

                let cache = cache_c.clone();
                let client = client_c.clone();
                let inflight = inflight_c.clone();
                let per_host = per_host_c.clone();
                let permit = permits_c.clone().acquire_owned().await.unwrap();

                tokio::spawn(async move {
                    let _p = permit; // holds concurrency slot
                    Self::handle_request(cache, client, inflight, per_host, req).await;
                });
            }
        });

        Self {
            cache,
            q_crit: q_crit_tx,
            q_high: q_high_tx,
            q_norm: q_norm_tx,
            q_low: q_low_tx,
            inflight,
            per_host,
            client,
        }
    }

    /// Start downloading an image in the background
    pub fn preload_image(&self, url: String, event_id: Uuid, priority: Priority) {
        let request = ImageDownloadRequest {
            url,
            event_id,
            priority,
            callback: None,
        };
        
        let tx = match request.priority {
            Priority::Critical => &self.q_crit,
            Priority::High => &self.q_high,
            Priority::Normal => &self.q_norm,
            Priority::Low => &self.q_low,
        };
        
        if let Err(e) = tx.try_send(request) {
            warn!("Queue full, dropping preload: {}", e);
        }
    }

    /// Download image immediately and return result
    pub async fn download_image_sync(&self, url: String, event_id: Uuid) -> Result<Bytes, ImageError> {
        // Check cache first
        if let Some(cached) = self.get_cached_image(&url).await {
            return Ok(cached);
        }

        // Create oneshot channel for result
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        let request = ImageDownloadRequest {
            url,
            event_id,
            priority: Priority::High,
            callback: Some(tx),
        };
        
        self.q_high.send(request).await
            .map_err(|_| ImageError::Cancelled)?;
        
        // Wait for download to complete
        rx.await
            .map_err(|_| ImageError::Cancelled)?
    }

    /// Get image from cache if available (read-only fast path)
    pub async fn get_cached_image(&self, url: &str) -> Option<Bytes> {
        if let Some(entry) = self.cache.get(url).await {
            entry.access_count.fetch_add(1, Ordering::Relaxed);
            Some(entry.data.clone())
        } else {
            None
        }
    }

    /// Check if image is cached
    pub async fn is_cached(&self, url: &str) -> bool {
        self.cache.contains_key(url)
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let entry_count = self.cache.entry_count();
        let bytes = self.cache.weighted_size(); // total bytes now
        
        CacheStats {
            entries: entry_count,
            total_size_bytes: bytes,
            total_size_mb: bytes as f64 / 1024.0 / 1024.0,
        }
    }

    /// Handle request with deduplication and coalescing
    async fn handle_request(
        cache: Cache<String, CacheEntry>,
        client: Client,
        inflight: Arc<Mutex<HashMap<String, Vec<tokio::sync::oneshot::Sender<Result<Bytes, ImageError>>>>>>,
        per_host: Arc<DashMap<String, Arc<Semaphore>>>,
        req: ImageDownloadRequest,
    ) {
        // Check cache first
        if let Some(entry) = cache.get(&req.url).await {
            entry.access_count.fetch_add(1, Ordering::Relaxed);
            if let Some(cb) = req.callback {
                let _ = cb.send(Ok(entry.data.clone()));
            }
            return;
        }

        // Coalesce in-flight downloads
        let mut inflight_guard = inflight.lock().await;
        if let Some(waiters) = inflight_guard.get_mut(&req.url) {
            if let Some(cb) = req.callback {
                waiters.push(cb);
            }
            return;
        } else {
            let mut waiters = Vec::new();
            if let Some(cb) = req.callback {
                waiters.push(cb);
            }
            inflight_guard.insert(req.url.clone(), waiters);
        }
        drop(inflight_guard);

        // Get per-host semaphore for concurrency control
        let host = Self::host_for(&req.url);
        let host_sem = per_host.entry(host).or_insert_with(|| Arc::new(Semaphore::new(4))).clone();
        let _host_permit = host_sem.acquire_owned().await.unwrap();

        // Perform download with priority-based timeout
        let deadline = Self::deadline_for(&req.priority);
        let result = tokio::time::timeout(deadline, Self::download_image(&client, &req.url))
            .await
            .unwrap_or(Err(ImageError::Timeout));

        // Store result and notify all waiters
        if let Ok(ref bytes) = result {
            let entry = CacheEntry {
                data: bytes.clone(),
                timestamp: chrono::Utc::now(),
                access_count: Arc::new(AtomicU32::new(1)),
            };
            cache.insert(req.url.clone(), entry).await;
        }

        // Log event_id for tracing
        match &result {
            Ok(b) => info!(url=%req.url, event=%req.event_id, bytes=b.len(), "cached image"),
            Err(e) => warn!(url=%req.url, event=%req.event_id, err=?e, "image fetch failed"),
        }

        let mut inflight_guard = inflight.lock().await;
        if let Some(waiters) = inflight_guard.remove(&req.url) {
            for cb in waiters {
                let _ = cb.send(result.clone());
            }
        }
    }

    // Content validation helper
    fn looks_like_image(b: &[u8]) -> bool {
        let png = b.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        let jpg = b.starts_with(&[0xFF, 0xD8, 0xFF]);
        let gif = b.starts_with(b"GIF8");
        let riff_webp = b.len() > 12 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP";
        png || jpg || gif || riff_webp
    }

    // Helper function to preserve timeout semantics
    fn map_net_error(e: reqwest::Error) -> ImageError {
        if e.is_timeout() {
            ImageError::Timeout
        } else {
            ImageError::Network(e.to_string())
        }
    }

    // Per-request deadline by priority
    fn deadline_for(p: &Priority) -> Duration {
        match p {
            Priority::Critical => Duration::from_secs(2),
            Priority::High => Duration::from_secs(4),
            Priority::Normal => Duration::from_secs(6),
            Priority::Low => Duration::from_secs(8),
        }
    }

    // Extract host from URL for per-host concurrency
    fn host_for(url: &str) -> String {
        Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_default()
    }

    // Allow application/octet-stream if magic bytes look like an image
    fn ct_allows_image(ct: &str) -> bool {
        ct.starts_with("image/") || ct == "application/octet-stream"
    }

    async fn download_image(client: &Client, url: &str) -> Result<Bytes, ImageError> {
        // HEAD request to check content type and size
        if let Ok(head) = client.head(url).send().await {
            if !head.status().is_success() {
                return Err(ImageError::HttpStatus { status: head.status().as_u16() });
            }
            
            // Check content length
            if let Some(cl) = head.headers().get(reqwest::header::CONTENT_LENGTH) {
                if let Ok(size_str) = cl.to_str() {
                    if let Ok(size) = size_str.parse::<usize>() {
                        if size > MAX_BYTES {
                            return Err(ImageError::TooLarge(size));
                        }
                    }
                }
            }
            
            // Check content type
            if let Some(ct) = head.headers().get(reqwest::header::CONTENT_TYPE) {
                if let Ok(content_type) = ct.to_str() {
                    if !Self::ct_allows_image(content_type) {
                        return Err(ImageError::UnsupportedContentType(content_type.to_string()));
                    }
                }
            }
        }

        // GET with Range header for initial validation
        let mut resp = client.get(url)
            .header(reqwest::header::RANGE, format!("bytes=0-{}", RANGE_BYTES - 1))
            .send().await
            .map_err(Self::map_net_error)?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ImageError::HttpStatus { status: status.as_u16() });
        }

        // Also validate content-type on GET path (when HEAD is skipped or wrong)
        if let Some(ct) = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()) {
            if !Self::ct_allows_image(ct) {
                return Err(ImageError::UnsupportedContentType(ct.to_string()));
            }
        }

        let content_range = resp.headers().get(reqwest::header::CONTENT_RANGE);
        let prefix = resp.bytes().await.map_err(Self::map_net_error)?;

        // If server ignored Range (200 without Content-Range), this is the full body
        if status == reqwest::StatusCode::OK && content_range.is_none() {
            if prefix.len() > MAX_BYTES {
                return Err(ImageError::TooLarge(prefix.len()));
            }
            if !Self::looks_like_image(&prefix) {
                return Err(ImageError::InvalidFormat);
            }
            return Ok(prefix);
        }
        
        if prefix.len() < RANGE_BYTES {
            // Likely got full body already
            if !Self::looks_like_image(&prefix) {
                return Err(ImageError::InvalidFormat);
            }
            return Ok(prefix);
        }

        // Validate image format from prefix
        if !Self::looks_like_image(&prefix) {
            return Err(ImageError::InvalidFormat);
        }

        // Fetch full content (bounded)
        let full = client.get(url).send().await
            .map_err(Self::map_net_error)?;
        
        if !full.status().is_success() {
            return Err(ImageError::HttpStatus { status: full.status().as_u16() });
        }
        
        // Validate content-type on full request too
        if let Some(ct) = full.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()) {
            if !Self::ct_allows_image(ct) {
                return Err(ImageError::UnsupportedContentType(ct.to_string()));
            }
        }
        
        let full_bytes = full.bytes().await
            .map_err(Self::map_net_error)?;
        
        if full_bytes.len() > MAX_BYTES {
            return Err(ImageError::TooLarge(full_bytes.len()));
        }
        
        Ok(full_bytes)
    }

    /// Extract image URLs from various data formats
    pub fn extract_image_urls(data: &str) -> Vec<String> {
        let mut urls = Vec::new();
        
        // Try parsing as JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
            Self::extract_urls_from_json(&json, &mut urls);
        }
        
        // Also look for direct URLs in text
        Self::extract_urls_from_text(data, &mut urls);
        
        urls
    }
    
    fn extract_urls_from_json(value: &serde_json::Value, urls: &mut Vec<String>) {
        match value {
            serde_json::Value::String(s) => {
                if Self::is_image_url(s) {
                    urls.push(s.clone());
                }
            }
            serde_json::Value::Object(map) => {
                for (_, v) in map {
                    Self::extract_urls_from_json(v, urls);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    Self::extract_urls_from_json(v, urls);
                }
            }
            _ => {}
        }
    }
    
    fn extract_urls_from_text(text: &str, urls: &mut Vec<String>) {
        // Simple regex-like URL extraction
        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            if Self::is_image_url(word) {
                urls.push(word.to_string());
            }
        }
    }
    
    fn is_image_url(s: &str) -> bool {
        let s = s.to_ascii_lowercase();
        (s.starts_with("http://") || s.starts_with("https://")) &&
        (s.ends_with(".jpg") || s.ends_with(".jpeg") || s.ends_with(".png") ||
         s.ends_with(".gif") || s.ends_with(".webp") || s.ends_with(".bmp"))
    }
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub entries: u64,
    pub total_size_bytes: u64,
    pub total_size_mb: f64,
}

// Helper function to extract single image URL from event data (for pipeline compatibility)
pub fn extract_image_url(data: &str) -> Option<String> {
    ImagePreloader::extract_image_urls(data).into_iter().next()
}

impl Default for ImagePreloader {
    fn default() -> Self {
        Self::new()
    }
}
