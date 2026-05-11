use std::time::Instant;

use suzu_app::{GameConfig, SuzuApp};

fn main() -> anyhow::Result<()> {
    let launched_without_args = std::env::args_os().len() == 1;
    let iterations = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1_000);
    let script = stress_script(iterations);
    let mut app = SuzuApp::new(GameConfig::default());

    let compile_start = Instant::now();
    app.load_script(&script)?;
    let compile_elapsed = compile_start.elapsed();

    let run_start = Instant::now();
    let mut steps = 0usize;
    while app.advance_script() {
        app.tick(16);
        steps += 1;
    }
    let run_elapsed = run_start.elapsed();

    println!("suzu-bench iterations={iterations}");
    println!("compiled_commands={steps}");
    println!("compile_ms={:.3}", compile_elapsed.as_secs_f64() * 1000.0);
    println!("run_ms={:.3}", run_elapsed.as_secs_f64() * 1000.0);
    println!(
        "commands_per_second={:.2}",
        steps as f64 / run_elapsed.as_secs_f64().max(f64::EPSILON)
    );
    if launched_without_args {
        pause_for_double_click();
    }
    Ok(())
}

fn pause_for_double_click() {
    #[cfg(windows)]
    {
        println!();
        println!("Press Enter to close...");
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    }
}

fn stress_script(iterations: usize) -> String {
    let mut script = String::from("@script version=1\n@bg file=\"stress/bg.png\"\n");
    for index in 0..iterations {
        script.push_str(&format!(
            "@char name=c{index} face=neutral x={} y={} layer={}\n",
            80 + index % 960,
            40 + index % 520,
            index % 16
        ));
        if index % 10 == 0 {
            script.push_str(&format!("# Narrator\nStress line {index}[l]continued\n"));
        }
    }
    script
}
