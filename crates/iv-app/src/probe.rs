//! Renderer probe run before the event loop exists (research R4).
//!
//! eframe cannot fall back from wgpu to glow after a failed start, and a winit event loop cannot
//! be created twice, so the choice is made up front from the adapters wgpu can enumerate.

use eframe::wgpu;

/// Adapter class, ordered from most to least preferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeviceKind {
    DiscreteGpu,
    IntegratedGpu,
    VirtualGpu,
    Cpu,
    Other,
}

impl DeviceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DiscreteGpu => "discrete",
            Self::IntegratedGpu => "integrated",
            Self::VirtualGpu => "virtual",
            Self::Cpu => "cpu",
            Self::Other => "other",
        }
    }

    fn from_wgpu(t: wgpu::DeviceType) -> Self {
        match t {
            wgpu::DeviceType::DiscreteGpu => Self::DiscreteGpu,
            wgpu::DeviceType::IntegratedGpu => Self::IntegratedGpu,
            wgpu::DeviceType::VirtualGpu => Self::VirtualGpu,
            wgpu::DeviceType::Cpu => Self::Cpu,
            wgpu::DeviceType::Other => Self::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterSummary {
    pub name: String,
    pub kind: DeviceKind,
    pub backend: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RendererChoice {
    Wgpu(AdapterSummary),
    Glow,
}

/// Picks the renderer: the best wgpu adapter by class, or glow when wgpu found none.
pub fn select_renderer(adapters: &[AdapterSummary]) -> RendererChoice {
    adapters
        .iter()
        .min_by_key(|a| a.kind)
        .cloned()
        .map_or(RendererChoice::Glow, RendererChoice::Wgpu)
}

fn summarize(info: &wgpu::AdapterInfo) -> AdapterSummary {
    AdapterSummary {
        name: info.name.clone(),
        kind: DeviceKind::from_wgpu(info.device_type),
        backend: format!("{:?}", info.backend).to_ascii_lowercase(),
    }
}

/// Enumerates adapters for the platform's primary backends without creating a window.
pub fn probe() -> RendererChoice {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::PRIMARY));
    let summaries: Vec<AdapterSummary> =
        adapters.iter().map(|a| summarize(&a.get_info())).collect();
    select_renderer(&summaries)
}

/// Adapter selector handed to egui-wgpu so the window uses the same ordering as the probe.
pub fn adapter_selector() -> eframe::egui_wgpu::NativeAdapterSelectorMethod {
    std::sync::Arc::new(
        |adapters: &[wgpu::Adapter], _surface: Option<&wgpu::Surface<'_>>| {
            adapters
                .iter()
                .min_by_key(|a| DeviceKind::from_wgpu(a.get_info().device_type))
                .cloned()
                .ok_or_else(|| "no wgpu adapter".to_string())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{AdapterSummary, DeviceKind, RendererChoice, select_renderer};

    fn adapter(name: &str, kind: DeviceKind) -> AdapterSummary {
        AdapterSummary {
            name: name.to_string(),
            kind,
            backend: "dx12".to_string(),
        }
    }

    #[test]
    fn no_adapters_selects_glow() {
        assert_eq!(select_renderer(&[]), RendererChoice::Glow);
    }

    #[test]
    fn prefers_discrete_then_integrated_then_virtual_then_cpu() {
        let cpu = adapter("Microsoft Basic Render Driver", DeviceKind::Cpu);
        let virt = adapter("virtio", DeviceKind::VirtualGpu);
        let igpu = adapter("Intel UHD", DeviceKind::IntegratedGpu);
        let dgpu = adapter("RTX", DeviceKind::DiscreteGpu);
        let other = adapter("mystery", DeviceKind::Other);

        let all = [
            other.clone(),
            cpu.clone(),
            virt.clone(),
            igpu.clone(),
            dgpu.clone(),
        ];
        assert_eq!(select_renderer(&all), RendererChoice::Wgpu(dgpu));
        let no_d = [cpu.clone(), virt.clone(), igpu.clone()];
        assert_eq!(select_renderer(&no_d), RendererChoice::Wgpu(igpu));
        let no_i = [cpu.clone(), virt.clone()];
        assert_eq!(select_renderer(&no_i), RendererChoice::Wgpu(virt));
        let only_cpu = [other.clone(), cpu.clone()];
        assert_eq!(select_renderer(&only_cpu), RendererChoice::Wgpu(cpu));
    }

    #[test]
    fn chosen_adapter_keeps_name_kind_and_backend_for_the_record() {
        let a = adapter("Apple M3", DeviceKind::IntegratedGpu);
        let RendererChoice::Wgpu(chosen) = select_renderer(std::slice::from_ref(&a)) else {
            panic!("expected wgpu");
        };
        assert_eq!(
            (
                chosen.name.as_str(),
                chosen.kind.as_str(),
                chosen.backend.as_str()
            ),
            ("Apple M3", "integrated", "dx12")
        );
    }
}
