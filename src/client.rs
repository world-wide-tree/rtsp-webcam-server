use opencv::highgui;
use opencv::videoio;
use opencv::prelude::*;

fn main(){
    let mut cam = videoio::VideoCapture::from_file_def("http://0.0.0.0:8787/").expect("Failed to get video capture");
    let mut frame = Mat::default();
    loop {
        cam.read(&mut frame).unwrap();
        highgui::imshow("ClientWindow", &frame).unwrap();
        if ('q' as i32) == highgui::wait_key(1).unwrap(){
            break;
        }
    }
}