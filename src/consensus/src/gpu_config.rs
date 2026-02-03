//! GPU Configuration and Detection for AequiHash
//!
//! Auto-detects and optimizes for ANY graphics card
//! - RTX series (20xx, 30xx, 40xx)
//! - AMD RDNA series (RX 6000-7000)
//! - Intel Arc series
//! - Integrated Intel/AMD graphics
//! - Legacy GPU support

use std::arch::x86_64;

/// GPU capabilities and optimal settings
#[derive(Debug, Clone)]
pub struct GpuConfig {
    /// GPU manufacturer and model
    pub gpu_name: String,

    /// Available VRAM in MB
    pub vram_mb: u32,

    /// Optimal batch size for parallel processing
    pub optimal_batch_size: u32,

    /// SIMD instruction support
    pub supports_avx2: bool,
    pub supports_avx512: bool,
    pub supports_sse4_1: bool,

    /// Memory access patterns
    pub supports_wide_simd: bool,
    pub cache_line_size: u32,

    /// Compute unit count
    pub compute_units: u32,

    /// Clock speed optimization
    pub memory_bandwidth_mbps: u32,
}

impl GpuConfig {
    /// Auto-detect GPU capabilities and optimal settings
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self::detect_x86_gpu()
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            Self::detect_generic_gpu()
        }
    }

    /// Detect GPU on x86_64 systems
    #[cfg(target_arch = "x86_64")]
    fn detect_x86_gpu() -> Self {
        let mut config = Self {
            gpu_name: "Unknown GPU".to_string(),
            vram_mb: 2048, // Conservative default
            optimal_batch_size: 256,
            supports_avx2: is_x86_feature_detected!("avx2"),
            supports_avx512: is_x86_feature_detected!("avx512f"),
            supports_sse4_1: is_x86_feature_detected!("sse4_1"),
            supports_wide_simd: is_x86_feature_detected!("avx2"),
            cache_line_size: 64,
            compute_units: 8,              // Conservative
            memory_bandwidth_mbps: 256000, // 256 GB/s default
        };

        // Try to detect GPU manufacturer through system info
        if let Ok(gpu_info) = Self::read_gpu_info() {
            config = Self::configure_for_gpu(&mut config, &gpu_info);
        } else {
            // Fallback: optimize based on CPU SIMD capabilities
            config = Self::optimize_by_cpu_features(config);
        }

        config
    }

    /// Detect GPU on non-x86 systems
    #[cfg(not(target_arch = "x86_64"))]
    fn detect_generic_gpu() -> Self {
        Self {
            gpu_name: "Generic GPU".to_string(),
            vram_mb: 2048,
            optimal_batch_size: 128,
            supports_avx2: false,
            supports_avx512: false,
            supports_sse4_1: false,
            supports_wide_simd: false,
            cache_line_size: 64,
            compute_units: 4,
            memory_bandwidth_mbps: 128000,
        }
    }

    /// Read GPU information from system (Windows/Linux/macOS)
    fn read_gpu_info() -> Result<String, ()> {
        #[cfg(target_os = "windows")]
        {
            Self::read_windows_gpu_info()
        }

        #[cfg(target_os = "linux")]
        {
            Self::read_linux_gpu_info()
        }

        #[cfg(target_os = "macos")]
        {
            Self::read_macos_gpu_info()
        }
    }

    /// Read Windows GPU information
    #[cfg(target_os = "windows")]
    fn read_windows_gpu_info() -> Result<String, ()> {
        // Try WMI for GPU detection
        use std::process::Command;

        let output = Command::new("wmic")
            .args(&["path", "win32_VideoController", "get", "name"])
            .output();

        if let Ok(output) = output {
            let gpu_name = String::from_utf8_lossy(&output.stdout)
                .lines()
                .nth(1) // Skip header
                .unwrap_or("Unknown")
                .trim();

            Ok(gpu_name.to_string())
        } else {
            Err(())
        }
    }

    /// Read Linux GPU information
    #[cfg(target_os = "linux")]
    fn read_linux_gpu_info() -> Result<String, ()> {
        use std::process::Command;

        // Try lspci or lshw
        if let Ok(output) = Command::new("lspci").args(&["-v", "-s", "VGA"]).output() {
            let gpu_info = String::from_utf8_lossy(&output.stdout);

            // Extract GPU name from lspci output
            for line in gpu_info.lines() {
                if line.contains("VGA compatible controller") || line.contains("3D controller") {
                    if let Some(name_start) = line.find(": ") {
                        if let Some(name_end) = line[name_start + 2..].find("\"") {
                            let gpu_name =
                                line[name_start + 2..name_start + 2 + name_end].to_string();
                            return Ok(gpu_name);
                        }
                    }
                }
            }
        }

        Err(())
    }

    /// Read macOS GPU information
    #[cfg(target_os = "macos")]
    fn read_macos_gpu_info() -> Result<String, ()> {
        use std::process::Command;

        if let Ok(output) = Command::new("system_profiler")
            .args(&["SPDisplaysDataType", "GPUPU"])
            .output()
        {
            let gpu_info = String::from_utf8_lossy(&output.stdout);

            // Parse GPU model from system profiler output
            for line in gpu_info.lines() {
                if line.contains("Chipset Model:") {
                    if let Some(name_start) = line.find(": ") {
                        if let Some(name_end) = line[name_start + 2..].find("\"") {
                            let gpu_name =
                                line[name_start + 2..name_start + 2 + name_end].to_string();
                            return Ok(gpu_name);
                        }
                    }
                }
            }
        }

        Err(())
    }

    /// Configure optimal settings based on detected GPU
    fn configure_for_gpu(base_config: &mut GpuConfig, gpu_name: &str) -> Self {
        let gpu_name_lower = gpu_name.to_lowercase();

        base_config.gpu_name = gpu_name.to_string();

        // NVIDIA RTX series optimization
        if gpu_name_lower.contains("rtx") {
            if gpu_name_lower.contains("40") {
                base_config.vram_mb = 16384; // 16GB+
                base_config.optimal_batch_size = 1024;
                base_config.compute_units = 128;
                base_config.memory_bandwidth_mbps = 1008000; // ~1000 GB/s
            } else if gpu_name_lower.contains("30") {
                base_config.vram_mb = 12288; // 12GB
                base_config.optimal_batch_size = 512;
                base_config.compute_units = 82; // RTX 3080 has 82 CUs
                base_config.memory_bandwidth_mbps = 760800; // ~760 GB/s
            } else if gpu_name_lower.contains("20") {
                base_config.vram_mb = 8192; // 8GB
                base_config.optimal_batch_size = 256;
                base_config.compute_units = 64; // RTX 2080 has 64 CUs
                base_config.memory_bandwidth_mbps = 616000; // ~616 GB/s
            }
        }
        // AMD RDNA/RDNA optimization
        else if gpu_name_lower.contains("radeon") || gpu_name_lower.contains("rx ") {
            if gpu_name_lower.contains("7") {
                base_config.vram_mb = 12288; // RX 7000 series
                base_config.optimal_batch_size = 384;
                base_config.compute_units = 60; // RDNA 3
                base_config.memory_bandwidth_mbps = 624000;
            } else if gpu_name_lower.contains("6") {
                base_config.vram_mb = 12288; // RX 6000 series
                base_config.optimal_batch_size = 320;
                base_config.compute_units = 40; // RDNA 2
                base_config.memory_bandwidth_mbps = 512000;
            }
        }
        // Intel Arc optimization
        else if gpu_name_lower.contains("arc") {
            base_config.vram_mb = 16384; // Arc A770
            base_config.optimal_batch_size = 256;
            base_config.compute_units = 32; // Arc A series
            base_config.memory_bandwidth_mbps = 560000;
        }
        // Integrated graphics optimization
        else if gpu_name_lower.contains("intel") && !gpu_name_lower.contains("arc") {
            base_config.vram_mb = 4096; // Shared memory estimate
            base_config.optimal_batch_size = 64;
            base_config.compute_units = 24; // Intel integrated
            base_config.memory_bandwidth_mbps = 102400; // ~100 GB/s
        }

        base_config.clone()
    }

    /// Optimize based on CPU SIMD features
    fn optimize_by_cpu_features(base_config: GpuConfig) -> GpuConfig {
        let mut config = base_config;

        if is_x86_feature_detected!("avx512f") {
            config.optimal_batch_size *= 4;
            config.supports_wide_simd = true;
        } else if is_x86_feature_detected!("avx2") {
            config.optimal_batch_size *= 2;
            config.supports_wide_simd = true;
        }

        config
    }

    /// Get optimization hints for the detected GPU
    pub fn optimization_hints(&self) -> String {
        format!(
            "🎮 GPU Configuration:\n\
             • GPU: {}\n\
             • VRAM: {}MB\n\
             • Batch Size: {}\n\
             • SIMD: AVX2={} AVX512={}\n\
             • Compute Units: {}\n\
             • Memory Bandwidth: {} MB/s\n\
             • Cache Line: {} bytes\n\
             \n\
             ⚡ Optimizations Applied:\n\
             • GPU-parallel memory mixing\n\
             • SIMD instruction utilization\n\
             • Cache-friendly access patterns\n\
             • Auto-detection of optimal parameters",
            self.gpu_name,
            self.vram_mb,
            self.optimal_batch_size,
            self.supports_avx2,
            self.supports_avx512,
            self.compute_units,
            self.memory_bandwidth_mbps,
            self.cache_line_size
        )
    }

    /// Trust-based fallback for ANY GPU
    pub fn trust_fallback() -> Self {
        Self {
            gpu_name: "Any GPU".to_string(),
            vram_mb: 4096,           // Conservative assumption
            optimal_batch_size: 128, // Safe for any hardware
            supports_avx2: is_x86_feature_detected!("avx2"),
            supports_avx512: false,
            supports_sse4_1: is_x86_feature_detected!("sse4_1"),
            supports_wide_simd: is_x86_feature_detected!("avx2"),
            cache_line_size: 64,
            compute_units: 8,              // Conservative
            memory_bandwidth_mbps: 256000, // Conservative estimate
        }
    }
}
