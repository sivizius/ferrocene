#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// remove later
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::
{
  io::
  {
    self,
    Read,
    Stdin,
    Stdout,
    Write,
  },
  os::
  {
    unix::
    {
      io::
      {
        AsRawFd,
      },
    },
  },
  thread,
  time,
};

mod yatui;

fn main()
{
  let mut myTUI                         = yatui::Yatui::new();
  let ( width, height, myTerminal )
  = myTUI.addTTYDisplay
    (
      yatui::display::Display_None,
      0,                                0,
      0,                                0,
      Box::new(io::stdin()),            Box::new(io::stdout()),
      10_000_000
    );
  let theStatusBar
  = myTUI.addStatusFrame
    (
      yatui::Frame_None,
      0,
      "?!".to_string(),
      '_',
    );
  let theImage
  = myTUI.addPixelFrame
    (
      0,                                0,
      0,                                0,
      1.0,
      yatui::Colour::Black,
      yatui::PixelEncoding::None,       yatui::PixelEncoding::None,
    );

  let theEditor
  = myTUI.addEditorFrame
    (
      yatui::Frame_None,
      0,                                0,
      vec!
      (
        vec!
        (
          yatui::StyledToken::new ( "1".to_string(),   yatui::Style_None,            0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "2".to_string(),  yatui::Style_None,            0, yatui::Colour::Blue,               yatui::Colour::Red              ),
        ),
        vec!
        (
          yatui::StyledToken::new ( "3".to_string(),   yatui::Style_None,            0, yatui::Colour::Default,            yatui::Colour::Default          ),
          yatui::StyledToken::new ( "4".to_string(),  yatui::Style_None,            0, yatui::Colour::Default,            yatui::Colour::Default          ),
        ),
        vec!
        (
          yatui::StyledToken::new ( "5".to_string(),   yatui::Style_None,            0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "6".to_string(),   yatui::Style_Italic,          0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "7".to_string(),   yatui::Style_Underline,       0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "8".to_string(),   yatui::Style_SlowBlink,       0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "9".to_string(),   yatui::Style_RapidBlink,      0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "0".to_string(),   yatui::Style_Inverse,         0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "a".to_string(),   yatui::Style_Conceal,         0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "b".to_string(),   yatui::Style_Fraktur,         0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "c".to_string(),   yatui::Style_DoubleUnderline, 0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "d".to_string(),   yatui::Style_Framed,          0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "e".to_string(),   yatui::Style_Encircled,       0, yatui::Colour::Green,              yatui::Colour::Black            ),
          yatui::StyledToken::new ( "f".to_string(),   yatui::Style_Overlined,       0, yatui::Colour::Green,              yatui::Colour::Black            ),
        ),
        vec!
        (
          yatui::StyledToken::new ( "g".to_string(),   yatui::Style_None,            0, yatui::Colour::FaintGreen,         yatui::Colour::FaintBlue        ),
          yatui::StyledToken::new ( "h".to_string(),  yatui::Style_None,            0, yatui::Colour::Green,              yatui::Colour::Blue             ),
        ),
        vec!
        (
          yatui::StyledToken::new ( "i".to_string(),     yatui::Style_None,            0, yatui::Colour::LightGreen,         yatui::Colour::Yellow           ),
          yatui::StyledToken::new ( "j".to_string(),      yatui::Style_None,            0, yatui::Colour::RGB( 255, 127, 0 ), yatui::Colour::Black            ),
          yatui::StyledToken::new ( "k".to_string(),    yatui::Style_None,            0, yatui::Colour::Grey(12),           yatui::Colour::Grey(23)         ),
        ),
        vec!
        (
          yatui::StyledToken::new ( "l".to_string(),     yatui::Style_None,            0, yatui::Colour::Standard(4),        yatui::Colour::Cube( 1, 2, 3 )  ),
          yatui::StyledToken::new ( "m".to_string(),     yatui::Style_None,            0, yatui::Colour::Bright(5),          yatui::Colour::Bright(2)        ),
        ),
      ),
      '.',
    );
  let theScreen
  = myTUI.addParentFrame
    (
      yatui::frame::Tiling::Grid,
      vec!
      (
        yatui::frame::Frame::newInstance
        (
          theEditor,
          0,                            0,
          width - 0,                    height - 1,
          width - 0,                    height - 1,
          width - 0,                    height - 1,
          0,                            0,
          1,                            1,
        ),
        yatui::frame::Frame::newInstance
        (
          theStatusBar,
          0,                            height as isize - 1,
          width,                        1,
          width,                        1,
          width,                        1,
          0,                            1,
          1,                            1,
        ),
      ),
      vec!(0, width as isize - 20),
      vec!(0, height as isize - 1, height as isize),
      vec!(1),
      vec!(1, 1),
      0
    );
  
  println!("turnOnDisplay");
  myTUI.turnOnDisplay
  (
    myTerminal,
    theScreen,
  ).unwrap();

  'mainLoop:
    loop
    {
      for event                         in                                      &myTUI.recvChannel.try_recv()
      {
        match event.event
        {
          yatui::event::EventType::Error(ref message) =>
          {
            println!("FAIL: {}", message);
          },
          yatui::event::EventType::Char(character) =>
          {
            if character == 'q'
            {
              break 'mainLoop;
            }
            println!("CHAR: {}", character);
          },
          _ => {}
        }
      }
      myTUI.render();
    }

  println!("turnOffDisplay");
  myTUI.turnOffDisplay
  (
    myTerminal,
  ).unwrap();
  
  println!("Wait for it………");
  thread::sleep(time::Duration::from_millis(5000));
  println!("turnOnDisplay again");
  myTUI.turnOnDisplay
  (
    myTerminal,
    theScreen,
  ).unwrap();

  'mainLoop1:
    loop
    {
      for event                         in                                      &myTUI.recvChannel.try_recv()
      {
        match event.event
        {
          yatui::event::EventType::Error(ref message) =>
          {
            println!("FAIL: {}", message);
          },
          yatui::event::EventType::Char(character) =>
          {
            if character == 'q'
            {
              break 'mainLoop1;
            }
            println!("CHAR: {}", character);
          },
          _ => {}
        }
      }
      myTUI.render();
    }
  myTUI.turnOffDisplay
  (
    myTerminal,
  ).unwrap();
  println!("turnOffDisplay again");
}
