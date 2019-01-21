#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#![cfg(feature = "display-tty")]

use ferrocene::
{
  Ferrocene,
  display::
  {
    DisplayFlag,
  },
  event::
  {
    EventType,
  },
  frame::
  {
    Frame,
    FrameFlag,
    PixelEncoding,
    Tiling,
    style::
    {
      Colour,
      Style,
      StyledToken,
    },
  },
};
use std::
{
  io::
  {
    self,
  },
  thread,
  time,
};

#[test]
fn main()
{

  let mut myTUI                         = Ferrocene::new();
  let ( width, height, myTerminal )
  = myTUI.addTTYDisplay
    (
      DisplayFlag::None,
      0,                                0,
      0,                                0,
      Box::new(io::stdin()),            Box::new(io::stdout()),
      10_000_000
    );
  let theStatusBar
  = myTUI.addStatusFrame
    (
      FrameFlag::None,
      0,
      "?!".to_string(),
      '_',
    );
  let _theImage
  = myTUI.addPixelFrame
    (
      0,                                0,
      0,                                0,
      1.0,
      Colour::Black,
      PixelEncoding::None,       PixelEncoding::None,
    );

  let theEditor
  = myTUI.addEditorFrame
    (
      FrameFlag::None,
      0,                                0,
      vec!
      (
        vec!
        (
          StyledToken::new ( "1".to_string(),   Style::None,            0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "2".to_string(),   Style::None,            0, Colour::Blue,               Colour::Red              ),
        ),
        vec!
        (
          StyledToken::new ( "A".to_string(),   Style::None,            0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,            1, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,            2, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,            3, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,            4, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,            5, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,            6, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,            7, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,            8, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,            9, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           10, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           11, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           12, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           13, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           14, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           15, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           16, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           17, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           18, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "A".to_string(),   Style::None,           19, Colour::Green,              Colour::Black            ),
        ),
        vec!
        (
          StyledToken::new ( "3".to_string(),   Style::None,            0, Colour::Default,            Colour::Default          ),
          StyledToken::new ( "4".to_string(),   Style::None,            0, Colour::Default,            Colour::Default          ),
        ),
        vec!
        (
          StyledToken::new ( "5".to_string(),   Style::None,            0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "6".to_string(),   Style::Italic,          0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "7".to_string(),   Style::Underline,       0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "8".to_string(),   Style::SlowBlink,       0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "9".to_string(),   Style::RapidBlink,      0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "0".to_string(),   Style::Inverse,         0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "a".to_string(),   Style::Conceal,         0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "b".to_string(),   Style::Fraktur,         0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "c".to_string(),   Style::DoubleUnderline, 0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "d".to_string(),   Style::Framed,          0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "e".to_string(),   Style::Encircled,       0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "f".to_string(),   Style::Overlined,       0, Colour::Green,              Colour::Black            ),
        ),
        vec!
        (
          StyledToken::new ( "g".to_string(),   Style::None,            0, Colour::FaintGreen,         Colour::FaintBlue        ),
          StyledToken::new ( "h".to_string(),   Style::None,            0, Colour::Green,              Colour::Blue             ),
        ),
        vec!
        (
          StyledToken::new ( "i".to_string(),   Style::None,            0, Colour::LightGreen,         Colour::Yellow           ),
          StyledToken::new ( "j".to_string(),   Style::None,            0, Colour::RGB( 255, 127, 0 ), Colour::Black            ),
          StyledToken::new ( "k".to_string(),   Style::None,            0, Colour::Grey(12),           Colour::Grey(23)         ),
        ),
        vec!
        (
          StyledToken::new ( "l".to_string(),   Style::None,            0, Colour::Standard(4),        Colour::Cube( 1, 2, 3 )  ),
          StyledToken::new ( "m".to_string(),   Style::None,            0, Colour::Bright(5),          Colour::Bright(2)        ),
        ),
      ),
      '.',
    );
  let theScreen
  = myTUI.addParentFrame
    (
      Tiling::Grid,
      vec!
      (
        Frame::newInstance
        (
          theEditor,
          0,                            0,
          width - 0,                    height - 1,
          width - 0,                    height - 1,
          width - 0,                    height - 1,
          0,                            0,
          1,                            1,
        ),
        Frame::newInstance
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
    "Hello Foobar".to_string()
  ).unwrap();

  'mainLoop:
    loop
    {
      for event                         in                                      &myTUI.recvChannel.try_recv()
      {
        match event.event
        {
          EventType::Error(ref message) =>
          {
            println!("FAIL: {}", message);
          },
          EventType::Character(character) =>
          {
            match character
            {
              'q' =>
              {
                break 'mainLoop;
              },
              'a' =>
              {
                myTUI.setDisplayTitle(myTerminal, "trolololololo".to_string());
              },
              _ => {}
            }
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
    "Hello World".to_string()
  ).unwrap();

  'mainLoop1:
    loop
    {
      for event                         in                                      &myTUI.recvChannel.try_recv()
      {
        match event.event
        {
          EventType::Error(ref message) =>
          {
            println!("FAIL: {}", message);
          },
          EventType::Character(character) =>
          {
            match character
            {
              'q' =>
              {
                break 'mainLoop1;
              },
              'a' =>
              {
                myTUI.setDisplayTitle(myTerminal, "trolololololo".to_string());
              },
              _ => {}
            }
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
