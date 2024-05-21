use crate::{Options, RustGPUShader};
use std::borrow::Cow;

pub struct CompiledShaderModules {
    named_spv_modules: Vec<(Option<String>, wgpu::ShaderModuleDescriptorSpirV<'static>)>,
}

impl CompiledShaderModules {
    pub fn spv_module_for_entry_point<'a>(
        &'a self,
        wanted_entry: &str,
    ) -> wgpu::ShaderModuleDescriptorSpirV<'a> {
        for (name, spv_module) in &self.named_spv_modules {
            match name {
                Some(name) if name != wanted_entry => continue,
                _ => {
                    return wgpu::ShaderModuleDescriptorSpirV {
                        label: name.as_deref(),
                        source: Cow::Borrowed(&spv_module.source),
                    };
                }
            }
        }
        unreachable!(
            "{wanted_entry:?} not found in modules {:?}",
            self.named_spv_modules
                .iter()
                .map(|(name, _)| name)
                .collect::<Vec<_>>()
        );
    }
}

pub fn maybe_watch(
    options: &Options,
    #[cfg(not(target_arch = "wasm32"))] on_watch: Option<
        Box<dyn FnMut(CompiledShaderModules) + Send + 'static>,
    >,
) -> CompiledShaderModules {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use spirv_builder::{CompileResult, MetadataPrintout, SpirvBuilder};
        use std::path::PathBuf;
        // Hack: spirv_builder builds into a custom directory if running under cargo, to not
        // deadlock, and the default target directory if not. However, packages like `proc-macro2`
        // have different configurations when being built here vs. when building
        // rustc_codegen_spirv normally, so we *want* to build into a separate target directory, to
        // not have to rebuild half the crate graph every time we run. So, pretend we're running
        // under cargo by setting these environment variables.
        std::env::set_var(
            "OUT_DIR",
            option_env!("SHADERS_TARGET_DIR").unwrap_or(env!("OUT_DIR")),
        );
        std::env::set_var("PROFILE", env!("PROFILE"));
        let crate_name = match options.shader {
            RustGPUShader::Mandelbrot => "mandelbrot",
            RustGPUShader::RayMarching => "ray-marching",
            RustGPUShader::RayMarching2D => "ray-marching-2d",
            RustGPUShader::SierpinskiTriangle => "sierpinski-triangle",
            RustGPUShader::KochSnowflake => "koch-snowflake",
            RustGPUShader::SDFs2D => "sdfs-2d",
            RustGPUShader::SDFs3D => "sdfs-3d",
            RustGPUShader::HydrogenWavefunction => "hydrogen-wavefunction",
            RustGPUShader::SphericalHarmonics => "spherical-harmonics",
            RustGPUShader::SphericalHarmonicsShape => "spherical-harmonics-shape",
            RustGPUShader::FunRepDemo => "fun-rep-demo",
            RustGPUShader::SdfBuilder => "sdf-builder",
            RustGPUShader::ProceduralGeneration => "procedural-generation",
        };
        let manifest_dir = option_env!("SHADERS_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR"));
        let crate_path = [manifest_dir, "..", "shaders", crate_name]
            .iter()
            .copied()
            .collect::<PathBuf>();

        let has_debug_printf = !options.validate_spirv;

        let builder = SpirvBuilder::new(crate_path, "spirv-unknown-vulkan1.1")
            .print_metadata(MetadataPrintout::None)
            .shader_panic_strategy(if has_debug_printf {
                spirv_builder::ShaderPanicStrategy::DebugPrintfThenExit {
                    print_inputs: true,
                    print_backtrace: true,
                }
            } else {
                spirv_builder::ShaderPanicStrategy::SilentExit
            })
            // HACK(eddyb) needed because of `debugPrintf` instrumentation limitations
            // (see https://github.com/KhronosGroup/SPIRV-Tools/issues/4892).
            .multimodule(has_debug_printf);
        let initial_result = if let Some(mut f) = on_watch {
            builder
                .watch(move |compile_result| f(handle_compile_result(compile_result)))
                .expect("Configuration is correct for watching")
        } else {
            builder.build().unwrap()
        };
        fn handle_compile_result(compile_result: CompileResult) -> CompiledShaderModules {
            let load_spv_module = |path| {
                let data = std::fs::read(path).unwrap();
                // FIXME(eddyb) this reallocates all the data pointlessly, there is
                // not a good reason to use `ShaderModuleDescriptorSpirV` specifically.
                let spirv = Cow::Owned(wgpu::util::make_spirv_raw(&data).into_owned());
                wgpu::ShaderModuleDescriptorSpirV {
                    label: None,
                    source: spirv,
                }
            };
            CompiledShaderModules {
                named_spv_modules: match compile_result.module {
                    spirv_builder::ModuleResult::SingleModule(path) => {
                        vec![(None, load_spv_module(path))]
                    }
                    spirv_builder::ModuleResult::MultiModule(modules) => modules
                        .into_iter()
                        .map(|(name, path)| (Some(name), load_spv_module(path)))
                        .collect(),
                },
            }
        }
        handle_compile_result(initial_result)
    }
}
