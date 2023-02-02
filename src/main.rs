use std::thread;

use anyhow::Result;

use webcam_viewer::UserEvent;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowBuilder;
use zenoh::prelude::r#async::AsyncResolve;
use zenoh::prelude::Encoding;
use zenoh::prelude::KnownEncoding;
use zenoh::prelude::SplitBuffer;

fn main() -> Result<()> {
    /*
        // Create a new capture device with a few extra parameters
        let mut dev = Device::new(0).expect("Failed to open device");

        // Let's say we want to explicitly request another format
        let mut fmt = dev.format().expect("Failed to read format");
        fmt.width = 1280;
        fmt.height = 720;
        fmt.fourcc = FourCC::new(b"MJPG");
        dev.set_format(&fmt).expect("Failed to write format");

        // The actual format chosen by the device driver may differ from what we
        // requested! Print it out to get an idea of what is actually used now.
        println!("Format in use:\n{}", fmt);
        // Create the stream, which will internally 'allocate' (as in map) the
        // number of requested buffers for us.
        let mut stream = Stream::with_buffers(&mut dev, Type::VideoCapture, 4)
            .expect("Failed to create buffer stream");
    */
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1600, 900))
        .with_resizable(true)
        .with_title("Webcam viewer")
        .build(&event_loop)?;

    thread::spawn(move || {
        async_std::task::block_on(async move {
            let config = zenoh::config::peer();
            let session = zenoh::open(config).res().await.unwrap();

            let subscriber = session
                .declare_subscriber("camera_node")
                .res()
                .await
                .unwrap();

            loop {
                let buf = subscriber.recv_async().await.unwrap();
                let encoding = buf
                    .value
                    .encoding(Encoding::Exact(KnownEncoding::AppOctetStream));
                let image = encoding.payload.contiguous();
                proxy
                    .send_event(UserEvent::NewImage(image.to_vec()))
                    .unwrap();
            }
        });
    });
    /*
        thread::spawn(move || loop {
            let buf = stream.next().unwrap().0;
            if !buf.is_empty() {
                proxy.send_event(UserEvent::NewImage(buf.to_vec())).unwrap();
            }
        });
    */
    pollster::block_on(webcam_viewer::run(event_loop, window));
    Ok(())
}
