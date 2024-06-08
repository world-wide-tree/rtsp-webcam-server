use std::time::Duration;

use opencv::{
    core::{Mat, Vector}, highgui, imgcodecs, prelude::*, videoio::{self, VideoCapture}
};
use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}, signal, sync::mpsc::{channel, Receiver, Sender}, task::JoinHandle};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Занимем хост...
    let listener = TcpListener::bind("0.0.0.0:8787").await?;
    println!("Server listening on port 8787");
    // Готовим канал для добовление подписчиков к потоку стримеру
    let (sub_ss, sub_rr) = channel(20);
    println!("Start read camera...");

    // Начинаем стриминг поток...
    let stream_handler = streaming_camera_task(sub_rr).await;
    if stream_handler.is_finished() {
        return stream_handler.await?.map_err(|e|e.into());
        // return Err("Camera proccess ended...")
    }
    // Вектор для сбора синхранизаторов потока...
    let mut join_handlers = Vec::new();
    // Получаем новые клиенты к серверу
    while let Ok(stream) = listener.accept().await{
        println!("New client on {}", stream.1);

        // Канал для получение данных
        let (sub_s, sub_r) = channel(100);
        // Подписка к стримеру
        if let Err(e) = sub_ss.send(sub_s).await{
            eprintln!("{e}");
        } else {
            // Если всё в порядке, то добавляем синхранизатору обработчик
            join_handlers.push(subscriber_task(stream.0, sub_r).await);
        }
    } 

    stream_handler.await??;
    for jh in join_handlers{
        jh.await??;
    }
    Ok(())
}

async fn subscriber_task(mut stream: TcpStream, mut frame_r: Receiver<Vector<u8>>) -> JoinHandle<Result<(), String>> {
    let handler = tokio::spawn(async move {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
        );
        if let Err(e) = stream.write_all(response.as_bytes()).await {
            return Err(format!("Failed to write response: {}", e));
        }
        loop {
            if let Some(buf) = frame_r.recv().await{
                let image_data = format!(
                    "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                    buf.len()
                );
    
                if let Err(e) = stream.write_all(image_data.as_bytes()).await {
                    let msg = format!("Failed to write image data: {}", e);
                    return Err(msg);
                }
                if let Err(e) = stream.write_all(buf.as_slice()).await {
                    let msg = format!("Failed to write image buffer: {}", e);
                    return Err(msg);
                }
                if let Err(e) = stream.write_all(b"\r\n").await {
                    let msg = format!("Failed to write image end: {}", e);
                    return Err(msg);
                }
                if let Err(e) = stream.flush().await {
                    let msg = format!("Failed to flush stream: {}", e);
                    return Err(msg);
                }
            }
            // tokio::select! {
            //     _ = signal::ctrl_c() => {
            //         break;
            //     }
            // }
        }
        Ok(())
    });
    handler
}

async fn streaming_camera_task(mut subscribe: Receiver<Sender<Vector<u8>>>) -> JoinHandle<Result<(), String>>{
    let h: JoinHandle<Result<(), String>> = tokio::spawn(async move {
        let mut cap = VideoCapture::from_file_def("rtsp://admin:123qazwsx@192.168.1.64:554/Streaming/channels/102").map_err(|e| e.to_string())?;
        if !cap.is_opened().map_err(|e| e.to_string())?{
            return Err("VideoCapture not opened!".into());
        } else {
            println!("Camera opened!");
        }
        let mut subscribers = Vec::new();
        loop {
            let mut buf = Vector::new();
            let mut frame = Mat::default();
            
            if !cap.read(&mut frame).map_err(|e| e.to_string())? || frame.size().map_err(|e| e.to_string())?.width <= 0 {
                return Err("Failed to read frame".into());
            }
            
            let _ = imgcodecs::imencode_def(".jpg", &frame, &mut buf);
            
            if let Ok(subscriber) = subscribe.try_recv(){
                subscribers.push(subscriber);
            }
            subscribers.retain(|subscriber|{
                if let Err(e) = subscriber.try_send(buf.clone()){
                    eprintln!("Frame send error: {e}. Close channel.");
                    false
                } else {
                    true
                }
            });
            // subscribers.iter_mut().for_each(|subscriber| {
            //     if subscriber.1{
            //         if let Err(e) = subscriber.0.try_send(buf.clone()){
            //             subscriber.1 = false;
            //             eprintln!("Frame send error: {e}");
            //         }
            //     } else {
            //         subscriber.
            //     }
            // });
            highgui::imshow("ServerWindow", &frame).map_err(|e| e.to_string())?;
            highgui::wait_key(1).map_err(|e| e.to_string())?;
            tokio::time::sleep(Duration::from_millis(30)).await;
            // tokio::select! {
            //     _ = signal::ctrl_c() => {
            //         break;
            //     }
            // }
        }
        Ok(())
    });
    h
}
