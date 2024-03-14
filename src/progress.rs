pub fn report(message: &str) {
    eprintln!("{}", message);
}

pub fn report_progress(percentage: f64) {
    eprint!("\rProgress: {:.2}%", percentage * 100.0);
    if percentage == 1.0 {
        eprintln!()
    }
}
