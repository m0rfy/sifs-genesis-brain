//! Agent loop, HTTP serve, compute_sifs stdin

use std::io::Write;

use crate::fixed::{f32_to_fixed, fixed_to_f32, sifs_threshold, sifs_w};
use crate::neuron::{calculate_sifs_current, BrainConfig};
use crate::constants::SIFS_LEVELS;
use crate::runner::BrainRunner;

pub fn run_agent_loop(config: BrainConfig) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, BufRead};

    let mut runner = BrainRunner::new(config.clone());
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    while let Some(line_result) = lines.next() {
        let line = line_result.map_err(|e| format!("stdin: {}", e))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut episode_end = false;
        let obs: Vec<f32> = match serde_json::from_str::<serde_json::Value>(line) {
            Ok(serde_json::Value::Array(arr)) => arr
                .into_iter()
                .filter_map(|v| v.as_f64().map(|x| x as f32))
                .collect(),
            Ok(serde_json::Value::Object(map)) => {
                if let Some(reward) = map.get("reward").and_then(|v| v.as_f64()) {
                    runner.last_reward = reward as f32;
                }
                episode_end = map.get("episode_end").and_then(|v| v.as_bool()).unwrap_or(false);
                map.get("input")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_f64().map(|x| x as f32))
                            .collect()
                    })
                    .unwrap_or_default()
            }
            Ok(_) => Vec::new(),
            Err(_) => Vec::new(),
        };
        let (readout, action) = if obs.is_empty() {
            runner.step(&[0.0; 4])
        } else {
            runner.step(&obs)
        };
        if episode_end {
            runner.trigger_night_phase();
        }
        let out = format!(
            r#"{{"readout":{{"spike_count_total":{},"score":{},"time_step":{}}},"action":{}}}"#,
            readout.spike_count_total,
            readout.score,
            readout.time_step,
            action
        );
        writeln!(stdout, "{}", out)?;
        stdout.flush()?;
    }
    Ok(())
}

pub fn run_serve_loop(config: BrainConfig, bind: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;
    use std::net::TcpListener;

    let mut runner = BrainRunner::new(config);
    let listener = TcpListener::bind(bind)?;
    eprintln!("SIFS Brain HTTP server on http://{} (POST /step)", bind);

    for stream in listener.incoming().flatten() {
        let mut stream = stream;
        let mut buf = [0u8; 65536];
        let n = stream.read(&mut buf).unwrap_or(0);
        let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
        let body_start = req
            .find("\r\n\r\n")
            .or_else(|| req.find("\n\n"))
            .map(|p| p + 2)
            .unwrap_or(0);
        let body = if body_start > 0 && body_start < req.len() {
            req[body_start..].trim()
        } else {
            ""
        };
        let result = serde_json::from_str::<serde_json::Value>(body).ok().and_then(|json| {
            let input = json.get("input")?.as_array()?;
            let input_f32: Vec<f32> = input
                .iter()
                .filter_map(|v| v.as_f64().map(|x| x as f32))
                .collect();
            Some(runner.step(&input_f32))
        });
        let (readout, action) = match result {
            Some((r, a)) => (r, a),
            None => {
                let _ = write_response(&mut stream, 400, r#"{"error":"bad request"}"#);
                continue;
            }
        };
        let resp_body = format!(
            r#"{{"readout":{{"spike_count_total":{},"score":{},"time_step":{}}},"action":{}}}"#,
            readout.spike_count_total,
            readout.score,
            readout.time_step,
            action
        );
        let _ = write_response(&mut stream, 200, &resp_body);
    }
    Ok(())
}

pub fn write_response(
    stream: &mut std::net::TcpStream,
    status: u16,
    body: &str,
) -> std::io::Result<()> {
    let status_line = if status == 200 {
        "HTTP/1.1 200 OK"
    } else {
        "HTTP/1.1 400 Bad Request"
    };
    let h = format!(
        "{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status_line,
        body.len(),
        body
    );
    stream.write_all(h.as_bytes())
}

pub fn run_compute_sifs_stdin() {
    use std::io::{self, BufRead};
    let mut weights = [0; SIFS_LEVELS];
    let mut thresholds = [0; SIFS_LEVELS];
    for n in 0..SIFS_LEVELS {
        weights[n] = sifs_w(n);
        thresholds[n] = sifs_threshold(n, f32_to_fixed(0.02));
    }
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: f32 = match line.parse() {
            Ok(x) => x,
            Err(_) => continue,
        };
        let v_fixed = f32_to_fixed(v);
        let i_sifs = calculate_sifs_current(v_fixed, &weights, &thresholds, f32_to_fixed(40.0));
        println!("{}", fixed_to_f32(i_sifs));
    }
}
