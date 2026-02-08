use std::process::Command;

pub fn parse_cmd_string(s: &str) -> Vec<String> {
    s.split(" ")
        .filter(|it| (!it.contains('%') && !it.contains('@')))
        .map(|it| it.to_string())
        .collect()
}

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
