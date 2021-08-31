use anyhow::Result;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::Write;

fn main() -> Result<()> {
    sudo::escalate_if_needed().unwrap();

    let icds = std::fs::read_dir("/usr/share/vulkan/icd.d/");

    let mut options = vec![];

    for icd in icds? {
        let path = icd?.path();

        let value: serde_json::Value =
            serde_json::from_str(std::fs::read_to_string(path.clone())?.as_str())?;

        let library_path = &value["ICD"]["library_path"];
        let enabled = path.extension() != Some(&OsStr::new("disabled"));

        options.push((library_path.clone(), path.clone(), enabled));
    }

    options.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));

    for (idx, (opt, _, enabled)) in options.iter().enumerate() {
        println!(
            "{} {}. {}",
            if *enabled { ">> " } else { "   " },
            idx + 1,
            opt
        );
    }

    print!("Select one of the ICD's to enable or Q to quit: ");
    std::io::stdout().flush();

    let selected_idx;

    loop {
        let mut line = String::new();
        let stdin = std::io::stdin();
        stdin.lock().read_line(&mut line).unwrap();

        let line = line.trim();

        if line == "q" {
            return Ok(())
        }
        
        let opt : usize = line.trim().parse()?;
        if opt >= 1 && opt <= options.len() {
            selected_idx = opt - 1;
            break;
        } else {
            println!("Invalid selection");
        }
    }

    for (idx, (opt, path, enabled)) in options.iter().enumerate() {
        if idx != selected_idx {
            let mut disabled_path = path.clone();
            disabled_path.set_extension("disabled");

            std::fs::rename(path, disabled_path)?;
        } else {
            let mut enabled_path = path.clone();
            enabled_path.set_extension("json");

            std::fs::rename(path, enabled_path)?;
        }
    }

    println!("\n\nSelected:\n{}. {}", selected_idx + 1, options[selected_idx].0);

    Ok(())
}
