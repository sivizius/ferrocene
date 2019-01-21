#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#![cfg(feature = "display-tty")]

use ferrocene::
{
  Ferrocene,
  display::*,
  event::*,
  frame::*,
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
      Display_None,
      0,                                0,
      0,                                0,
      Box::new(io::stdin()),            Box::new(io::stdout()),
      10_000_000
    );
  let theStatusBar
  = myTUI.addStatusFrame
    (
      Frame_None,
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
      Frame_None,
      0,                                0,
      vec!
      (
        vec!
        (
          StyledToken::new ( "1".to_string(),   Style_None,            0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "2".to_string(),  Style_None,            0, Colour::Blue,               Colour::Red              ),
        ),
        vec!
        (
          StyledToken::new ( "3".to_string(),   Style_None,            0, Colour::Default,            Colour::Default          ),
          StyledToken::new ( "4".to_string(),  Style_None,            0, Colour::Default,            Colour::Default          ),
        ),
        vec!
        (
          StyledToken::new ( "5".to_string(),   Style_None,            0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "6".to_string(),   Style_Italic,          0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "7".to_string(),   Style_Underline,       0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "8".to_string(),   Style_SlowBlink,       0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "9".to_string(),   Style_RapidBlink,      0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "0".to_string(),   Style_Inverse,         0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "a".to_string(),   Style_Conceal,         0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "b".to_string(),   Style_Fraktur,         0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "c".to_string(),   Style_DoubleUnderline, 0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "d".to_string(),   Style_Framed,          0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "e".to_string(),   Style_Encircled,       0, Colour::Green,              Colour::Black            ),
          StyledToken::new ( "f".to_string(),   Style_Overlined,       0, Colour::Green,              Colour::Black            ),
        ),
        vec!
        (
          StyledToken::new ( "g".to_string(),   Style_None,            0, Colour::FaintGreen,         Colour::FaintBlue        ),
          StyledToken::new ( "h".to_string(),  Style_None,            0, Colour::Green,              Colour::Blue             ),
        ),
        vec!
        (
          StyledToken::new ( "i".to_string(),     Style_None,            0, Colour::LightGreen,         Colour::Yellow           ),
          StyledToken::new ( "j".to_string(),      Style_None,            0, Colour::RGB( 255, 127, 0 ), Colour::Black            ),
          StyledToken::new ( "k".to_string(),    Style_None,            0, Colour::Grey(12),           Colour::Grey(23)         ),
        ),
        vec!
        (
          StyledToken::new ( "l".to_string(),     Style_None,            0, Colour::Standard(4),        Colour::Cube( 1, 2, 3 )  ),
          StyledToken::new ( "m".to_string(),     Style_None,            0, Colour::Bright(5),          Colour::Bright(2)        ),
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
          EventType::Char(character) =>
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
          EventType::Error(ref message) =>
          {
            println!("FAIL: {}", message);
          },
          EventType::Char(character) =>
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
