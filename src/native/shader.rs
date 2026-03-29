mod rasterzation { 
    vulkano_shaders::shader!(
        ty: "compute",
        path: "src/shaders/rasterzation/rasterzation.comp"
    );
}