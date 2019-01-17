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
};

use termion::
{
  event::
  {
    Event,
    Key,
    MouseButton,
    MouseEvent,
  },
};

mod yatui;

fn main()
{
  let mut myTUI                         = yatui::Yatui::new();

  let ( width, height, theDisplay )
  = myTUI.newTermion
    (
      0,                                0,
      io::stdin(),                      io::stdout(),
      false,                            100_000_000,
    );

  let theStatusBar
  = myTUI.newStatusFrame
    (
      yatui::Frame_None,
      0,
      "Hello".to_string(),
      '_',
    );

  let theEditor
  = myTUI.newEditorFrame
    (
      yatui::Frame_None,
      0,                                0,
      vec!
      (
        vec!
        (
          yatui::YWord::new ( "Hello ".to_string(),   yatui::YWord_None,            0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "World! ".to_string(),  yatui::YWord_None,            0, yatui::YColour::Blue,         yatui::YColour::Red ),
        ),
        vec!
        (
          yatui::YWord::new ( "Hello ".to_string(),   yatui::YWord_None,            0, yatui::YColour::Default,      yatui::YColour::Default ),
          yatui::YWord::new ( "World! ".to_string(),  yatui::YWord_None,            0, yatui::YColour::Default,      yatui::YColour::Default ),
        ),
        vec!
        (
          yatui::YWord::new ( "Hello!".to_string(),   yatui::YWord_None,            0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello?".to_string(),   yatui::YWord_Italic,          0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello9".to_string(),   yatui::YWord_Underline,       0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello8".to_string(),   yatui::YWord_SlowBlink,       0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello7".to_string(),   yatui::YWord_RapidBlink,      0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello6".to_string(),   yatui::YWord_Inverse,         0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello5".to_string(),   yatui::YWord_Conceal,         0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello4".to_string(),   yatui::YWord_Fraktur,         0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello3".to_string(),   yatui::YWord_DoubleUnderline, 0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello2".to_string(),   yatui::YWord_Framed,          0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello1".to_string(),   yatui::YWord_Encircled,       0, yatui::YColour::Green,        yatui::YColour::Black ),
          yatui::YWord::new ( "Hello0".to_string(),   yatui::YWord_Overlined,       0, yatui::YColour::Green,        yatui::YColour::Black ),
        ),
        vec!
        (
          yatui::YWord::new ( "Hello ".to_string(),   yatui::YWord_None,            0, yatui::YColour::FaintGreen,   yatui::YColour::FaintBlue ),
          yatui::YWord::new ( "World! ".to_string(),  yatui::YWord_None,            0, yatui::YColour::Green,        yatui::YColour::Blue ),
        ),
        vec!
        (
          yatui::YWord::new ( "How ".to_string(),     yatui::YWord_None,            0, yatui::YColour::LightGreen,   yatui::YColour::Yellow ),
          yatui::YWord::new ( "ya ".to_string(),      yatui::YWord_None,            0, yatui::YColour::RGB( 255,     127, 0 ), yatui::YColour::Black ),
          yatui::YWord::new ( "doin?".to_string(),    yatui::YWord_None,            0, yatui::YColour::Grey(12),     yatui::YColour::Grey(23) ),
        ),
        vec!
        (
          yatui::YWord::new ( "foo ".to_string(),     yatui::YWord_None,            0, yatui::YColour::Standard(4),  yatui::YColour::Cube( 1, 2, 3 ) ),
          yatui::YWord::new ( "bar!".to_string(),     yatui::YWord_None,            0, yatui::YColour::Bright(5),    yatui::YColour::Bright(2) ),
        ),
      ),
      '.',
    );

  let theScreen
  = myTUI.newParentFrame
    (
      yatui::Tiling::None,
      vec!
      (
        yatui::Yatui::newInstance
        (
          theEditor,
          0,                            0,
          width - 0,                    height - 1,
          width - 0,                    height - 1,
          width - 0,                    height - 1,
          0,                            0,
          0,                            0,
        ),
        yatui::Yatui::newInstance
        (
          theStatusBar,
          0,                            height as isize - 1,
          width,                        1,
          width,                        1,
          width,                        1,
          0,                            0,
          0,                            0,
        ),
      ),
      vec!(),                           vec!(),
      0
    );

  myTUI.turnOnDisplay
  (
    theDisplay,
    theScreen,
  );

  'mainLoop:
    loop
    {
      for display                       in                                      &mut myTUI.listOfDisplays
      {
        if let Some(ref mut refDisplay) = display
        {
          match refDisplay.display
          {
            yatui::YDisplayType::Console(ref output) =>
            {
            },
            yatui::YDisplayType::Termion(ref display) =>
            {
              for event in &display.events.try_recv()
              {
                match event
                {
                  Event::Key(key)                                     =>
                  {
                    match key
                    {
                      Key::Esc                                        =>
                      {
                        break 'mainLoop;
                      },
                      Key::Char('f')                                  =>
                      {
                        let mut myStatusBar
                                        =                                       yatui::Yatui::swapFrame ( &mut myTUI.listOfFrames, theStatusBar, None ).unwrap();
                        if let Some(ref mut frame) = myStatusBar
                        {
                          if let yatui::YFrame::Status(ref mut status) = frame
                          {
                            status.text.push('A')
                          }
                        }
                        yatui::Yatui::swapFrame ( &mut myTUI.listOfFrames, theStatusBar, myStatusBar ).unwrap();
                        refDisplay.needRefresh
                                        =                                       true;
                      },
                      _                                               =>
                      {
                      }
                    }
                  },
                  Event::Mouse(MouseEvent::Press(button, posX, posY)) =>
                  {
                  },
                  Event::Mouse(MouseEvent::Release(posX, posY))       =>
                  {
                  },
                  Event::Mouse(MouseEvent::Hold(posX, posY))          =>
                  {
                  },
                  Event::Unsupported(u)                               =>
                  {
                  },
                }
              }
            }
          }
        }
      }
      myTUI.render();
    }
  myTUI.turnOffDisplay
  (
    theDisplay,
  );
}