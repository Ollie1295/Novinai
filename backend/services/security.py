#!/usr/bin/env python3
"""
Security Hardening Module
JWT validation, rate limiting, CORS, authentication, and security middleware
"""

import logging
import time
import hashlib
import hmac
import secrets
import json
import asyncio
from datetime import datetime, timedelta, timezone
from typing import Dict, List, Any, Optional, Tuple, Callable
from functools import wraps
import redis.asyncio as redis
from dataclasses import dataclass
import jwt
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
import base64
import os

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class SecurityConfig:
    """Security configuration"""
    jwt_secret: str
    jwt_algorithm: str = "HS256"
    jwt_expiry_hours: int = 24
    
    # Rate limiting
    rate_limit_per_minute: int = 100
    rate_limit_burst: int = 200
    
    # Signed URLs
    signed_url_secret: str = None
    signed_url_ttl_seconds: int = 3600  # 1 hour
    
    # CORS
    allowed_origins: List[str] = None
    allowed_methods: List[str] = None
    allowed_headers: List[str] = None
    
    # Security headers
    enable_security_headers: bool = True
    hsts_max_age: int = 31536000  # 1 year
    
    def __post_init__(self):
        if not self.jwt_secret:
            raise ValueError("JWT secret is required")
        
        if not self.signed_url_secret:
            self.signed_url_secret = self.jwt_secret
            
        if self.allowed_origins is None:
            self.allowed_origins = ["https://app.insane.ai", "https://api.insane.ai"]
            
        if self.allowed_methods is None:
            self.allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
            
        if self.allowed_headers is None:
            self.allowed_headers = [
                "Content-Type", "Authorization", "X-User-ID", 
                "X-Home-ID", "X-Request-ID", "X-API-Version"
            ]

class JWTManager:
    """JWT token management"""
    
    def __init__(self, config: SecurityConfig):
        self.config = config
        
    def generate_token(
        self, 
        user_id: str, 
        home_id: str = None,
        tier: str = "standard",
        permissions: List[str] = None
    ) -> str:
        """Generate JWT token"""
        try:
            now = datetime.now(timezone.utc)
            payload = {
                "user_id": user_id,
                "home_id": home_id,
                "tier": tier,
                "permissions": permissions or [],
                "iat": now,
                "exp": now + timedelta(hours=self.config.jwt_expiry_hours),
                "iss": "insane-ai-security",
                "aud": "mobile-api"
            }
            
            return jwt.encode(
                payload, 
                self.config.jwt_secret, 
                algorithm=self.config.jwt_algorithm
            )
            
        except Exception as e:
            logger.error(f"Failed to generate JWT token: {e}")
            raise
    
    def validate_token(self, token: str) -> Dict[str, Any]:
        """Validate JWT token and return payload"""
        try:
            payload = jwt.decode(
                token,
                self.config.jwt_secret,
                algorithms=[self.config.jwt_algorithm],
                audience="mobile-api",
                issuer="insane-ai-security"
            )
            
            # Additional validation
            required_fields = ["user_id", "tier"]
            for field in required_fields:
                if field not in payload:
                    raise jwt.InvalidTokenError(f"Missing required field: {field}")
            
            return payload
            
        except jwt.ExpiredSignatureError:
            logger.warning("JWT token expired")
            raise
        except jwt.InvalidTokenError as e:
            logger.warning(f"Invalid JWT token: {e}")
            raise
        except Exception as e:
            logger.error(f"JWT token validation failed: {e}")
            raise
    
    def refresh_token(self, token: str) -> str:
        """Refresh JWT token if close to expiry"""
        try:
            payload = self.validate_token(token)
            
            # Check if token is within 1 hour of expiry
            exp = datetime.fromtimestamp(payload['exp'], timezone.utc)
            now = datetime.now(timezone.utc)
            
            if exp - now < timedelta(hours=1):
                return self.generate_token(
                    user_id=payload['user_id'],
                    home_id=payload.get('home_id'),
                    tier=payload['tier'],
                    permissions=payload.get('permissions', [])
                )
            
            return token
            
        except Exception as e:
            logger.error(f"Token refresh failed: {e}")
            raise

class RateLimiter:
    """Redis-based rate limiter"""
    
    def __init__(self, redis_client, config: SecurityConfig):
        self.redis_client = redis_client
        self.config = config
        
    async def check_rate_limit(
        self, 
        identifier: str, 
        limit: int = None, 
        window_seconds: int = 60
    ) -> Tuple[bool, Dict[str, Any]]:
        """Check if request is within rate limit"""
        try:
            if limit is None:
                limit = self.config.rate_limit_per_minute
                
            key = f"rate_limit:{identifier}"
            current_time = int(time.time())
            window_start = current_time - window_seconds
            
            # Use sliding window rate limiting
            pipeline = self.redis_client.pipeline()
            
            # Remove old entries
            pipeline.zremrangebyscore(key, 0, window_start)
            
            # Count current requests
            pipeline.zcard(key)
            
            # Add current request
            pipeline.zadd(key, {str(current_time): current_time})
            
            # Set expiry
            pipeline.expire(key, window_seconds * 2)
            
            results = await pipeline.execute()
            current_count = results[1]
            
            # Check if over limit
            if current_count >= limit:
                # Remove the request we just added since it's rejected
                await self.redis_client.zrem(key, str(current_time))
                
                return False, {
                    "allowed": False,
                    "limit": limit,
                    "current": current_count,
                    "reset_time": current_time + window_seconds,
                    "retry_after": window_seconds
                }
            
            return True, {
                "allowed": True,
                "limit": limit,
                "current": current_count + 1,
                "remaining": limit - current_count - 1,
                "reset_time": current_time + window_seconds
            }
            
        except Exception as e:
            logger.error(f"Rate limit check failed: {e}")
            # Allow request on error to avoid blocking legitimate traffic
            return True, {"allowed": True, "error": str(e)}
    
    async def get_rate_limit_status(self, identifier: str, window_seconds: int = 60) -> Dict[str, Any]:
        """Get current rate limit status"""
        try:
            key = f"rate_limit:{identifier}"
            current_time = int(time.time())
            window_start = current_time - window_seconds
            
            # Clean old entries and count current
            await self.redis_client.zremrangebyscore(key, 0, window_start)
            current_count = await self.redis_client.zcard(key)
            
            return {
                "limit": self.config.rate_limit_per_minute,
                "current": current_count,
                "remaining": max(0, self.config.rate_limit_per_minute - current_count),
                "reset_time": current_time + window_seconds
            }
            
        except Exception as e:
            logger.error(f"Rate limit status check failed: {e}")
            return {"error": str(e)}

class SignedURLManager:
    """Signed URL generation and validation"""
    
    def __init__(self, config: SecurityConfig):
        self.config = config
        
    def generate_signed_url(
        self, 
        path: str, 
        expires_in: int = None,
        user_id: str = None,
        permissions: List[str] = None
    ) -> str:
        """Generate signed URL"""
        try:
            if expires_in is None:
                expires_in = self.config.signed_url_ttl_seconds
                
            expiry = int(time.time()) + expires_in
            
            # Create signature payload
            payload = {
                "path": path,
                "expiry": expiry,
                "user_id": user_id,
                "permissions": permissions or []
            }
            
            payload_str = json.dumps(payload, sort_keys=True)
            
            # Generate HMAC signature
            signature = hmac.new(
                self.config.signed_url_secret.encode(),
                payload_str.encode(),
                hashlib.sha256
            ).hexdigest()
            
            # Encode payload
            payload_b64 = base64.urlsafe_b64encode(payload_str.encode()).decode()
            
            return f"{path}?signature={signature}&payload={payload_b64}"
            
        except Exception as e:
            logger.error(f"Failed to generate signed URL: {e}")
            raise
    
    def validate_signed_url(self, path: str, signature: str, payload: str) -> Tuple[bool, Dict[str, Any]]:
        """Validate signed URL"""
        try:
            # Decode payload
            payload_str = base64.urlsafe_b64decode(payload.encode()).decode()
            payload_data = json.loads(payload_str)
            
            # Check expiry
            if time.time() > payload_data.get("expiry", 0):
                return False, {"error": "URL expired", "expired": True}
            
            # Verify path matches
            if payload_data.get("path") != path:
                return False, {"error": "Path mismatch"}
            
            # Verify signature
            expected_signature = hmac.new(
                self.config.signed_url_secret.encode(),
                payload_str.encode(),
                hashlib.sha256
            ).hexdigest()
            
            if not hmac.compare_digest(signature, expected_signature):
                return False, {"error": "Invalid signature"}
            
            return True, {
                "valid": True,
                "user_id": payload_data.get("user_id"),
                "permissions": payload_data.get("permissions", []),
                "expires_at": payload_data.get("expiry")
            }
            
        except Exception as e:
            logger.error(f"Signed URL validation failed: {e}")
            return False, {"error": str(e)}

class SecurityMiddleware:
    """Security middleware for request processing"""
    
    def __init__(
        self, 
        jwt_manager: JWTManager,
        rate_limiter: RateLimiter,
        signed_url_manager: SignedURLManager,
        config: SecurityConfig
    ):
        self.jwt_manager = jwt_manager
        self.rate_limiter = rate_limiter
        self.signed_url_manager = signed_url_manager
        self.config = config
        
    def get_client_identifier(self, request_data: Dict[str, Any]) -> str:
        """Get client identifier for rate limiting"""
        # Priority: user_id > ip_address > api_key > generic
        if request_data.get("user_id"):
            return f"user:{request_data['user_id']}"
        elif request_data.get("ip_address"):
            return f"ip:{request_data['ip_address']}"
        elif request_data.get("api_key"):
            return f"api:{hashlib.sha256(request_data['api_key'].encode()).hexdigest()[:16]}"
        else:
            return "anonymous"
    
    async def validate_request(self, request_data: Dict[str, Any]) -> Tuple[bool, Dict[str, Any]]:
        """Validate incoming request"""
        try:
            validation_result = {
                "valid": True,
                "user_info": None,
                "rate_limit": None,
                "errors": []
            }
            
            # Rate limiting check
            client_id = self.get_client_identifier(request_data)
            rate_ok, rate_info = await self.rate_limiter.check_rate_limit(client_id)
            validation_result["rate_limit"] = rate_info
            
            if not rate_ok:
                validation_result["valid"] = False
                validation_result["errors"].append("Rate limit exceeded")
                return False, validation_result
            
            # JWT validation (if token provided)
            auth_header = request_data.get("authorization", "")
            if auth_header.startswith("Bearer "):
                token = auth_header[7:]
                try:
                    payload = self.jwt_manager.validate_token(token)
                    validation_result["user_info"] = payload
                except jwt.InvalidTokenError as e:
                    validation_result["valid"] = False
                    validation_result["errors"].append(f"Invalid JWT: {e}")
                    return False, validation_result
            
            # Signed URL validation (if signature provided)
            if request_data.get("signature") and request_data.get("payload"):
                url_valid, url_info = self.signed_url_manager.validate_signed_url(
                    request_data.get("path", ""),
                    request_data["signature"],
                    request_data["payload"]
                )
                
                if not url_valid:
                    validation_result["valid"] = False
                    validation_result["errors"].append(f"Invalid signed URL: {url_info.get('error')}")
                    return False, validation_result
                
                # Merge URL permissions with JWT if both present
                if validation_result.get("user_info"):
                    url_perms = url_info.get("permissions", [])
                    jwt_perms = validation_result["user_info"].get("permissions", [])
                    validation_result["user_info"]["permissions"] = list(set(jwt_perms + url_perms))
            
            return True, validation_result
            
        except Exception as e:
            logger.error(f"Request validation failed: {e}")
            return False, {
                "valid": False,
                "errors": [f"Validation error: {e}"]
            }
    
    def get_cors_headers(self, origin: str = None) -> Dict[str, str]:
        """Get CORS headers"""
        headers = {}
        
        if origin and origin in self.config.allowed_origins:
            headers["Access-Control-Allow-Origin"] = origin
        elif "*" in self.config.allowed_origins:
            headers["Access-Control-Allow-Origin"] = "*"
        
        headers["Access-Control-Allow-Methods"] = ", ".join(self.config.allowed_methods)
        headers["Access-Control-Allow-Headers"] = ", ".join(self.config.allowed_headers)
        headers["Access-Control-Max-Age"] = "86400"  # 24 hours
        
        return headers
    
    def get_security_headers(self) -> Dict[str, str]:
        """Get security headers"""
        if not self.config.enable_security_headers:
            return {}
            
        return {
            "Strict-Transport-Security": f"max-age={self.config.hsts_max_age}; includeSubDomains",
            "X-Content-Type-Options": "nosniff",
            "X-Frame-Options": "DENY",
            "X-XSS-Protection": "1; mode=block",
            "Content-Security-Policy": "default-src 'self'; script-src 'self' 'unsafe-inline'",
            "Referrer-Policy": "strict-origin-when-cross-origin"
        }

class HealthCheckManager:
    """Health check and monitoring endpoints"""
    
    def __init__(self, redis_client):
        self.redis_client = redis_client
        
    async def check_health(self) -> Dict[str, Any]:
        """Comprehensive health check"""
        health_status = {
            "status": "healthy",
            "timestamp": datetime.utcnow().isoformat(),
            "checks": {},
            "version": "1.0.0"
        }
        
        try:
            # Redis connectivity check
            redis_start = time.time()
            await self.redis_client.ping()
            redis_duration = (time.time() - redis_start) * 1000
            
            health_status["checks"]["redis"] = {
                "status": "healthy",
                "response_time_ms": round(redis_duration, 2)
            }
            
        except Exception as e:
            health_status["status"] = "unhealthy"
            health_status["checks"]["redis"] = {
                "status": "unhealthy",
                "error": str(e)
            }
        
        # System resource checks
        try:
            import psutil
            
            # Memory usage
            memory = psutil.virtual_memory()
            health_status["checks"]["memory"] = {
                "status": "healthy" if memory.percent < 90 else "warning",
                "usage_percent": memory.percent,
                "available_gb": round(memory.available / (1024**3), 2)
            }
            
            # Disk usage
            disk = psutil.disk_usage('/')
            health_status["checks"]["disk"] = {
                "status": "healthy" if disk.percent < 90 else "warning",
                "usage_percent": disk.percent,
                "free_gb": round(disk.free / (1024**3), 2)
            }
            
            # CPU usage
            cpu_percent = psutil.cpu_percent(interval=1)
            health_status["checks"]["cpu"] = {
                "status": "healthy" if cpu_percent < 80 else "warning",
                "usage_percent": cpu_percent
            }
            
        except ImportError:
            health_status["checks"]["system"] = {
                "status": "unknown",
                "error": "psutil not available"
            }
        except Exception as e:
            health_status["checks"]["system"] = {
                "status": "error",
                "error": str(e)
            }
        
        # Overall status determination
        unhealthy_checks = [
            check for check in health_status["checks"].values()
            if check.get("status") == "unhealthy"
        ]
        
        if unhealthy_checks:
            health_status["status"] = "unhealthy"
        elif any(check.get("status") == "warning" for check in health_status["checks"].values()):
            health_status["status"] = "degraded"
        
        return health_status
    
    async def check_readiness(self) -> Dict[str, Any]:
        """Readiness probe for container orchestration"""
        try:
            # Check critical dependencies
            await self.redis_client.ping()
            
            return {
                "ready": True,
                "timestamp": datetime.utcnow().isoformat()
            }
            
        except Exception as e:
            return {
                "ready": False,
                "timestamp": datetime.utcnow().isoformat(),
                "error": str(e)
            }
    
    async def check_liveness(self) -> Dict[str, Any]:
        """Liveness probe for container orchestration"""
        return {
            "alive": True,
            "timestamp": datetime.utcnow().isoformat(),
            "uptime_seconds": time.time() - self.start_time if hasattr(self, 'start_time') else 0
        }

# Security decorators and utilities
def require_auth(permissions: List[str] = None):
    """Decorator to require authentication"""
    def decorator(func: Callable) -> Callable:
        @wraps(func)
        async def wrapper(request_data: Dict[str, Any], *args, **kwargs):
            # This would be implemented based on your web framework
            # Here's a generic example
            
            auth_header = request_data.get("authorization", "")
            if not auth_header.startswith("Bearer "):
                return {"error": "Authentication required", "status": 401}
            
            # JWT validation would happen here
            # You would inject the security middleware into your endpoint handlers
            
            return await func(request_data, *args, **kwargs)
        return wrapper
    return decorator

def require_permissions(required_permissions: List[str]):
    """Decorator to require specific permissions"""
    def decorator(func: Callable) -> Callable:
        @wraps(func)
        async def wrapper(request_data: Dict[str, Any], *args, **kwargs):
            user_info = request_data.get("user_info", {})
            user_permissions = user_info.get("permissions", [])
            
            if not all(perm in user_permissions for perm in required_permissions):
                return {"error": "Insufficient permissions", "status": 403}
            
            return await func(request_data, *args, **kwargs)
        return wrapper
    return decorator

async def main():
    """Test security components"""
    # Configuration
    config = SecurityConfig(
        jwt_secret=os.getenv("JWT_SECRET", "test-secret-key"),
        rate_limit_per_minute=10,
        allowed_origins=["http://localhost:3000"]
    )
    
    # Initialize Redis
    redis_client = redis.from_url("redis://localhost:6379", decode_responses=True)
    
    try:
        # Initialize components
        jwt_manager = JWTManager(config)
        rate_limiter = RateLimiter(redis_client, config)
        signed_url_manager = SignedURLManager(config)
        security_middleware = SecurityMiddleware(
            jwt_manager, rate_limiter, signed_url_manager, config
        )
        health_manager = HealthCheckManager(redis_client)
        
        print("=== Security Component Testing ===")
        
        # Test JWT
        print("\n1. Testing JWT:")
        token = jwt_manager.generate_token("user123", "home456", "premium", ["read", "write"])
        print(f"Generated token: {token[:50]}...")
        
        payload = jwt_manager.validate_token(token)
        print(f"Validated payload: {payload}")
        
        # Test rate limiting
        print("\n2. Testing rate limiting:")
        for i in range(3):
            allowed, info = await rate_limiter.check_rate_limit("test_user")
            print(f"Request {i+1}: allowed={allowed}, remaining={info.get('remaining')}")
        
        # Test signed URL
        print("\n3. Testing signed URL:")
        signed_url = signed_url_manager.generate_signed_url(
            "/api/events/123/image", 
            expires_in=300, 
            user_id="user123"
        )
        print(f"Signed URL: {signed_url}")
        
        # Test health check
        print("\n4. Testing health check:")
        health = await health_manager.check_health()
        print(f"Health status: {health['status']}")
        print(f"Redis check: {health['checks'].get('redis', {}).get('status')}")
        
    finally:
        await redis_client.close()

if __name__ == "__main__":
    asyncio.run(main())
