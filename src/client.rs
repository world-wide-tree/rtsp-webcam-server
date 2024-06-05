use opencv::highgui;
use opencv::videoio;
use opencv::prelude::*;
use opencv::imgcodecs;

fn main() {
    let mut cam = videoio::VideoCapture::from_file_def("http://0.0.0.0:8787/").expect("Failed to get video capture");
    let mut frame = Mat::default();
    loop {
        if !cam.read(&mut frame).unwrap() || frame.size().unwrap().width <= 0 {
            eprintln!("Failed to read frame");
            break;
        }

        // let img = imgcodecs::imdecode(&frame, imgcodecs::IMREAD_COLOR).unwrap();
        highgui::imshow("ClientWindow", &frame).unwrap();
        if ('q' as i32) == highgui::wait_key(35).unwrap() {
            break;
        }
    }
}
