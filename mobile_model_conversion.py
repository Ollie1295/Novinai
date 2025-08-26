#!/usr/bin/env python3
"""
Mobile Model Conversion for On-Device Inference
Converts YOLOv8 models to CoreML (iOS) and TensorFlow Lite (Android)
"""

import torch
from ultralytics import YOLO
import coremltools as ct
import tensorflow as tf
import os
import logging
from pathlib import Path

logger = logging.getLogger(__name__)

class MobileModelConverter:
    def __init__(self, model_path: str = "yolov8n.pt"):
        self.model_path = model_path
        self.model = YOLO(model_path)
        self.output_dir = Path("mobile_models")
        self.output_dir.mkdir(exist_ok=True)
    
    def convert_to_coreml(self, input_size: int = 384) -> str:
        """Convert YOLOv8 to CoreML for iOS"""
        try:
            logger.info(f"Converting {self.model_path} to CoreML (iOS)")
            
            # Export to CoreML with optimizations
            coreml_path = self.model.export(
                format="coreml",
                imgsz=input_size,
                int8=True,  # Use INT8 quantization for mobile
                nms=True,   # Include NMS in model
                simplify=True
            )
            
            # Move to mobile_models directory
            output_path = self.output_dir / f"yolo_lite_{input_size}.mlmodel"
            if os.path.exists(coreml_path):
                os.rename(coreml_path, output_path)
            
            logger.info(f"✅ CoreML model saved: {output_path}")
            return str(output_path)
            
        except Exception as e:
            logger.error(f"CoreML conversion failed: {e}")
            raise
    
    def convert_to_tflite(self, input_size: int = 384) -> str:
        """Convert YOLOv8 to TensorFlow Lite for Android"""
        try:
            logger.info(f"Converting {self.model_path} to TensorFlow Lite (Android)")
            
            # Export to TensorFlow Lite with optimizations
            tflite_path = self.model.export(
                format="tflite",
                imgsz=input_size,
                int8=True,  # Use INT8 quantization
                simplify=True
            )
            
            # Move to mobile_models directory
            output_path = self.output_dir / f"yolo_lite_{input_size}.tflite"
            if os.path.exists(tflite_path):
                os.rename(tflite_path, output_path)
            
            logger.info(f"✅ TensorFlow Lite model saved: {output_path}")
            return str(output_path)
            
        except Exception as e:
            logger.error(f"TensorFlow Lite conversion failed: {e}")
            raise
    
    def validate_mobile_model(self, model_path: str, format: str):
        """Validate converted mobile model"""
        try:
            if format == "coreml":
                # Load and validate CoreML model
                model = ct.models.MLModel(model_path)
                spec = model.get_spec()
                
                logger.info(f"CoreML model validation:")
                logger.info(f"  Input: {spec.description.input}")
                logger.info(f"  Output: {spec.description.output}")
                
            elif format == "tflite":
                # Load and validate TensorFlow Lite model
                interpreter = tf.lite.Interpreter(model_path=model_path)
                interpreter.allocate_tensors()
                
                input_details = interpreter.get_input_details()
                output_details = interpreter.get_output_details()
                
                logger.info(f"TensorFlow Lite model validation:")
                logger.info(f"  Input shape: {input_details[0]['shape']}")
                logger.info(f"  Output shape: {output_details[0]['shape']}")
            
            logger.info(f"✅ {format.upper()} model validation successful")
            
        except Exception as e:
            logger.error(f"{format.upper()} model validation failed: {e}")
            raise

def convert_models_for_mobile():
    """Convert YOLOv8 models for mobile deployment"""
    
    # Download YOLOv8n if not exists
    model_path = "yolov8n.pt"
    if not os.path.exists(model_path):
        logger.info("Downloading YOLOv8n model...")
        model = YOLO(model_path)  # This downloads the model
    
    converter = MobileModelConverter(model_path)
    
    # Convert for different input sizes
    input_sizes = [320, 384, 416]  # Mobile-optimized sizes
    
    for size in input_sizes:
        try:
            # Convert to CoreML (iOS)
            coreml_path = converter.convert_to_coreml(size)
            converter.validate_mobile_model(coreml_path, "coreml")
            
            # Convert to TensorFlow Lite (Android)
            tflite_path = converter.convert_to_tflite(size)
            converter.validate_mobile_model(tflite_path, "tflite")
            
        except Exception as e:
            logger.error(f"Failed to convert model for size {size}: {e}")
            continue
    
    # Generate model metadata
    metadata = {
        "models": {
            "ios": [f"yolo_lite_{size}.mlmodel" for size in input_sizes],
            "android": [f"yolo_lite_{size}.tflite" for size in input_sizes]
        },
        "input_sizes": input_sizes,
        "classes": [
            "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck",
            "boat", "traffic light", "fire hydrant", "stop sign", "parking meter", "bench",
            "bird", "cat", "dog", "horse", "sheep", "cow", "elephant", "bear", "zebra",
            "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee",
            "skis", "snowboard", "sports ball", "kite", "baseball bat", "baseball glove",
            "skateboard", "surfboard", "tennis racket", "bottle", "wine glass", "cup",
            "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange",
            "broccoli", "carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch",
            "potted plant", "bed", "dining table", "toilet", "tv", "laptop", "mouse",
            "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink",
            "refrigerator", "book", "clock", "vase", "scissors", "teddy bear", "hair drier",
            "toothbrush"
        ],
        "performance": {
            "target_inference_time_ms": 300,
            "max_input_size": 384,
            "quantization": "int8"
        }
    }
    
    import json
    with open(converter.output_dir / "model_metadata.json", "w") as f:
        json.dump(metadata, f, indent=2)
    
    logger.info("✅ Mobile model conversion complete")
    logger.info(f"Models saved in: {converter.output_dir}")

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    
    try:
        convert_models_for_mobile()
        print("🚀 Mobile models ready for deployment")
    except Exception as e:
        print(f"❌ Model conversion failed: {e}")
        exit(1)
