use chrono::{Local, NaiveDateTime, Duration, ParseError};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::sleep;
use std::time::{Duration as StdDuration};
use ctrlc;

fn main() {
    println!("Enter the alarm sound file path: ");
    let mut file_path = String::new();
    io::stdin().read_line(&mut file_path).expect("Failed to read line");
    let file_path = file_path.trim().to_string();
    
    let current_time = Local::now().naive_local();
    let formatted_time = current_time.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("Current time: {}", formatted_time);

    println!("Enter the alarm time in the format YYYY-MM-DD HH:MM:SS: ");
    let mut date_time = String::new();
    io::stdin().read_line(&mut date_time).expect("Failed to read line");
    let date_time = date_time.trim().to_string();


    match parse_date_time(&date_time) {
        Ok(parsed_time) => {
            if is_valid_date(&parsed_time, current_time) {
                println!("Parsed time: {}", parsed_time);

                match calculate_time(parsed_time, current_time) {
                    Some(duration) => {
                        println!(
                            "Alarm in: {} hours, {} minutes, and {} seconds",
                            duration.num_hours(),
                            duration.num_minutes() % 60,
                            duration.num_seconds() % 60
                        );

                        let alarm_seconds = duration.num_seconds();
                        for second in (0..alarm_seconds).rev() {
                            println!("{} seconds remaining...", second + 1);
                            sleep(StdDuration::from_millis(1000));
                        }

                        println!("Wake up!");
                        play_alarm(&file_path);
                    }
                    None => println!("Error: The alarm time must be in the future."),
                }
            } else {
                println!("Error: The alarm time must be today or tomorrow.");
            }
        }
        Err(e) => println!("Error parsing date and time: {}", e),
    }
}

fn parse_date_time(input: &str) -> Result<NaiveDateTime, ParseError> {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S")
}

fn is_valid_date(parsed_time: &NaiveDateTime, current_time: NaiveDateTime) -> bool {
    let current_date = current_time.date();
    let next_date = current_date + Duration::days(1);
    let input_date = parsed_time.date();

    if input_date == current_date {
        parsed_time > &current_time
    } else {
        input_date == next_date
    }
}

fn calculate_time(parsed_time: NaiveDateTime, current_time: NaiveDateTime) -> Option<Duration> {
    if parsed_time > current_time {
        Some(parsed_time - current_time)
    } else {
        None
    }
}

fn play_alarm(file_path: &str) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));

    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_clone = Arc::clone(&is_running);
    let sink_clone = Arc::clone(&sink);

    ctrlc::set_handler(move || {
        is_running_clone.store(false, Ordering::SeqCst);
        sink_clone.lock().unwrap().stop();
    }).expect("Error setting Ctrl-C handler");

    while is_running.load(Ordering::SeqCst) {
        let file = File::open(file_path).unwrap();
        let source = Decoder::new(file).unwrap();

        {
            let mut sink = sink.lock().unwrap();
            sink.append(source);
        }

        while !sink.lock().unwrap().empty() && is_running.load(Ordering::SeqCst) {
            sleep(StdDuration::from_millis(50));
        }

        if !is_running.load(Ordering::SeqCst) {
            break;
        }
    }
}