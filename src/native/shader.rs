pub mod miss {
    use vulkano_shaders::shader;

    shader!(
        bytes: "target/shaders/miss.spv"
    );
}
