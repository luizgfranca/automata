use std::process::Command;

pub fn try_run(cmd: &Vec<String>) {
    if let Some(app) = cmd.get(0) {
        let mut command = Command::new(app);
        let args = &cmd[1..];
        for it in args {
            command.arg(&it);
        }

        let output = command.spawn();
        match output {
            Err(e) => println!("unable to spawn process {}", e.to_string()),
            _ => (),
        }
    }
}
