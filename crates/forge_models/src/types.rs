//! # Model Types
//!
//! Type definitions for model formats and quantization schemes.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported model file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelFormat {
    /// GGUF format (GPT-Generated Unified Format)
    GGUF,
    /// SafeTensors format
    SafeTensors,
    /// PyTorch format (.pt, .pth)
    PyTorch,
    /// ONNX format
    ONNX,
    /// TensorFlow format
    TensorFlow,
    /// Other/unknown format
    Other,
}

impl fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelFormat::GGUF => write!(f, "gguf"),
            ModelFormat::SafeTensors => write!(f, "safetensors"),
            ModelFormat::PyTorch => write!(f, "pytorch"),
            ModelFormat::ONNX => write!(f, "onnx"),
            ModelFormat::TensorFlow => write!(f, "tensorflow"),
            ModelFormat::Other => write!(f, "other"),
        }
    }
}

impl FromStr for ModelFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gguf" => Ok(ModelFormat::GGUF),
            "safetensors" => Ok(ModelFormat::SafeTensors),
            "pytorch" | "pt" | "pth" => Ok(ModelFormat::PyTorch),
            "onnx" => Ok(ModelFormat::ONNX),
            "tensorflow" | "tf" => Ok(ModelFormat::TensorFlow),
            "other" => Ok(ModelFormat::Other),
            _ => Err(format!("Unknown model format: {}", s)),
        }
    }
}

/// Quantization type for model weights.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum QuantizationType {
    /// No quantization (full precision)
    None,
    /// 16-bit floating point
    F16,
    /// 8-bit integer quantization
    Q8_0,
    /// 4-bit K-means quantization (small)
    Q4_K_S,
    /// 4-bit K-means quantization (medium)
    Q4_K_M,
    /// 5-bit K-means quantization (small)
    Q5_K_S,
    /// 5-bit K-means quantization (medium)
    Q5_K_M,
    /// 6-bit K-means quantization
    Q6_K,
    /// 2-bit quantization
    Q2_K,
    /// 3-bit quantization
    Q3_K_S,
    /// 3-bit quantization (medium)
    Q3_K_M,
    /// 4-bit quantization (legacy)
    Q4_0,
    /// 4-bit quantization (legacy, improved)
    Q4_1,
    /// 5-bit quantization (legacy)
    Q5_0,
    /// 5-bit quantization (legacy, improved)
    Q5_1,
    /// Other/unknown quantization
    Other(String),
}

impl fmt::Display for QuantizationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantizationType::None => write!(f, "none"),
            QuantizationType::F16 => write!(f, "f16"),
            QuantizationType::Q8_0 => write!(f, "q8_0"),
            QuantizationType::Q4_K_S => write!(f, "q4_k_s"),
            QuantizationType::Q4_K_M => write!(f, "q4_k_m"),
            QuantizationType::Q5_K_S => write!(f, "q5_k_s"),
            QuantizationType::Q5_K_M => write!(f, "q5_k_m"),
            QuantizationType::Q6_K => write!(f, "q6_k"),
            QuantizationType::Q2_K => write!(f, "q2_k"),
            QuantizationType::Q3_K_S => write!(f, "q3_k_s"),
            QuantizationType::Q3_K_M => write!(f, "q3_k_m"),
            QuantizationType::Q4_0 => write!(f, "q4_0"),
            QuantizationType::Q4_1 => write!(f, "q4_1"),
            QuantizationType::Q5_0 => write!(f, "q5_0"),
            QuantizationType::Q5_1 => write!(f, "q5_1"),
            QuantizationType::Other(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for QuantizationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NONE" | "" => Ok(QuantizationType::None),
            "F16" => Ok(QuantizationType::F16),
            "Q8_0" => Ok(QuantizationType::Q8_0),
            "Q4_K_S" => Ok(QuantizationType::Q4_K_S),
            "Q4_K_M" => Ok(QuantizationType::Q4_K_M),
            "Q5_K_S" => Ok(QuantizationType::Q5_K_S),
            "Q5_K_M" => Ok(QuantizationType::Q5_K_M),
            "Q6_K" => Ok(QuantizationType::Q6_K),
            "Q2_K" => Ok(QuantizationType::Q2_K),
            "Q3_K_S" => Ok(QuantizationType::Q3_K_S),
            "Q3_K_M" => Ok(QuantizationType::Q3_K_M),
            "Q4_0" => Ok(QuantizationType::Q4_0),
            "Q4_1" => Ok(QuantizationType::Q4_1),
            "Q5_0" => Ok(QuantizationType::Q5_0),
            "Q5_1" => Ok(QuantizationType::Q5_1),
            _ => Ok(QuantizationType::Other(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_format_display() {
        assert_eq!(ModelFormat::GGUF.to_string(), "gguf");
        assert_eq!(ModelFormat::SafeTensors.to_string(), "safetensors");
        assert_eq!(ModelFormat::PyTorch.to_string(), "pytorch");
    }

    #[test]
    fn test_model_format_from_str() {
        assert_eq!("gguf".parse::<ModelFormat>().unwrap(), ModelFormat::GGUF);
        assert_eq!("GGUF".parse::<ModelFormat>().unwrap(), ModelFormat::GGUF);
        assert_eq!("pt".parse::<ModelFormat>().unwrap(), ModelFormat::PyTorch);
        assert_eq!("safetensors".parse::<ModelFormat>().unwrap(), ModelFormat::SafeTensors);
    }

    #[test]
    fn test_quantization_display() {
        assert_eq!(QuantizationType::Q4_K_M.to_string(), "q4_k_m");
        assert_eq!(QuantizationType::F16.to_string(), "f16");
        assert_eq!(QuantizationType::None.to_string(), "none");
    }

    #[test]
    fn test_quantization_from_str() {
        assert_eq!("Q4_K_M".parse::<QuantizationType>().unwrap(), QuantizationType::Q4_K_M);
        assert_eq!("q4_k_m".parse::<QuantizationType>().unwrap(), QuantizationType::Q4_K_M);
        assert_eq!("F16".parse::<QuantizationType>().unwrap(), QuantizationType::F16);
        assert_eq!("none".parse::<QuantizationType>().unwrap(), QuantizationType::None);
    }

    #[test]
    fn test_quantization_unknown() {
        match "CUSTOM_QUANT".parse::<QuantizationType>().unwrap() {
            QuantizationType::Other(s) => assert_eq!(s, "CUSTOM_QUANT"),
            _ => panic!("Expected Other variant"),
        }
    }
}
