use opencv::{
    core::{Mat, Vector}, highgui, imgcodecs, prelude::*, videoio
};

use std::net::TcpListener;
use std::io::Write;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8787").unwrap();
    println!("Server listening on port 8787");

    let mut cam = videoio::VideoCapture::from_file_def("rtsp://admin:123qazwsx@192.168.1.64:554/Streaming/channels/102").expect("Failed to get video capture");
    
    let mut frame = Mat::default();
    if !cam.is_opened().expect("Capture not opened!"){
        return;
    }
    // println!("Start streaming!");
    // loop {
    //     if !cam.read(&mut frame).unwrap() || frame.size().unwrap().width <= 0 {
    //         eprintln!("Failed to read frame");
    //         break;
    //     }

    //     // let img = imgcodecs::imdecode(&frame, imgcodecs::IMREAD_COLOR).unwrap();
    //     highgui::imshow("ClientWindow", &frame).unwrap();
    //     if ('q' as i32) == highgui::wait_key(1).unwrap() {
    //         break;
    //     }
    // }
    // println!("Error streaming!");
    for stream in listener.incoming() {
        println!("connected");
        let mut buf = Vector::new();
        let mut stream = stream.expect("Failed to accept connection");
        
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
        );
        
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Failed to write response: {}", e);
            continue;
        }
        
        loop {
            let mut frame = Mat::default();
            if !cam.read(&mut frame).expect("Failed to capture frame") || frame.size().unwrap().width <= 0 {
                //eprintln!("Failed to read frame from camera, frame: {:#?}", frame);
                //break;
            }
            buf.clear();
            let _ = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new());

            let image_data = format!(
                "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                buf.len()
            );

            if let Err(e) = stream.write_all(image_data.as_bytes()) {
                eprintln!("Failed to write image data: {}", e);
                break;
            }
            if let Err(e) = stream.write_all(buf.as_slice()) {
                eprintln!("Failed to write image buffer: {}", e);
                break;
            }
            if let Err(e) = stream.write_all(b"\r\n") {
                eprintln!("Failed to write image end: {}", e);
                break;
            }
            if let Err(e) = stream.flush() {
                eprintln!("Failed to flush stream: {}", e);
                break;
            }
            thread::sleep(Duration::from_millis(30)); // добавьте небольшую задержку для предотвращения перегрузки
        }
    }
}