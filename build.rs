use npm_rs::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // vite build は production用にビルドをしてくれる
    // 常にNODE_ENV = developmentで実行すればよい
    let exit_status = NpmEnv::default()
        // .with_node_env(&NodeEnv::from_cargo_profile().unwrap_or_default())
        .with_node_env(&NodeEnv::Development)
        .set_path("client")
        .with_env("FOO", "bar")
        .init_env()
        .install(None)
        .run("build")
        .exec()?;

    assert!(exit_status.success());

    Ok(())
}
