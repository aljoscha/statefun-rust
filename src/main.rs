use exitfailure::ExitFailure;

fn main() -> Result<(), ExitFailure> {
    env_logger::init();

    println!("Hello StateFun.");

    Ok(())
}
