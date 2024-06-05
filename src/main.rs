use opencv::{
    core::{Mat, Vector}, imgcodecs, prelude::*, videoio,
};

use std::net::TcpListener;
use std::io::Write;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8787").unwrap();
    println!("Server listening on port 8787");

    let mut cam = videoio::VideoCapture::from_file_def("rtsp://admin:123qazwsx@192.168.1.64:554/Streaming/channels/102").expect("Failed to get video capture");
    let mut frame = Mat::default();
    let mut buf = Vector::new();
    
    for stream in listener.incoming() {
        let mut stream = stream.expect("Failed to accept connection");
        
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
        );

        stream.write_all(response.as_bytes()).unwrap();
        
        loop {
            cam.read(&mut frame).expect("Failed to capture frame");
            buf.clear();
            let _ = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new());

            let image_data = format!(
                "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                buf.len()
            );

            stream.write_all(image_data.as_bytes()).unwrap();
            stream.write_all(buf.as_slice()).unwrap();
            stream.write_all(b"\r\n").unwrap();
            stream.flush().unwrap();
        }
    }

    // loop {
    //     let (mut stream, _) = listener.accept().expect("Failed to accept connection");
        
    //     cam.read(&mut frame).expect("Failed to capture frame");
    //     buf.clear();
    //     let _ = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new());

    //     let response = format!(
    //         "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
    //     );

    //     stream.write_all(response.as_bytes()).unwrap();
        
    //     loop {
    //         cam.read(&mut frame).expect("Failed to capture frame");
    //         buf.clear();
    //         let _ = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new());

    //         let image_data = format!(
    //             "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
    //             buf.len()
    //         );

    //         stream.write_all(image_data.as_bytes()).unwrap();
    //         stream.write_all(buf.as_slice()).unwrap();
    //         stream.write_all(b"\r\n").unwrap();
    //         stream.flush().unwrap();
    //     }
    // }
}
