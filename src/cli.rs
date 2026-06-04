use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // Optionally build example config
    #[cfg(debug_assertions)]
    #[arg(long)]
    pub build_example_config: bool
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_args_debug_assertions() {
        let terminal_args = vec!["program", "--build-example-config"];
        let result = Args::try_parse_from(terminal_args);

        #[cfg(debug_assertions)]
        {
            assert!(result.is_ok());
            let args = result.unwrap();
            assert!(args.build_example_config);
        }

        #[cfg(not(debug_assertions))]
        {
            assert!(result.is_err())
        }
    }
}