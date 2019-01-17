#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// remove later
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate termion;
pub mod terminal;
use std::
{
  io::
  {
    self,
    stdin,
    stdout,
    Write,
  },
  rc::
  {
    Rc,
    Weak,
  },
  sync::
  {
    mpsc::
    {
      channel,
      Receiver,
      Sender,
    },
    RwLock,
  },
  thread,
  time::
  {
    Duration,
    SystemTime
  },
};
use termion::
{
  async_stdin,
  clear,
  color,
  cursor,
  event::
  {
    Event,
    Key,
    MouseButton,
    MouseEvent,
  },
  input::
  {
    MouseTerminal,
    TermRead,
  },
  raw::
  {
    IntoRawMode,
  },
  style,
};


fn main ()
{
  let mut myTerminal                    = terminal::Terminal::new();
  /*let mut theStatusBar
  = terminal::TextBuffer::new
  (
    "",
    terminal::TextBufferNull
  );*/
  let     theStatusBar
  = terminal::BufferType::newTextBuffer
    (
      "",
      terminal::TextBufferNull
    );
  let     uidStatusBar
  = myTerminal.newBuffer
    (
      0,                        myTerminal.sizeY - 1,
      myTerminal.sizeX,         1,
      1,                        1,
      myTerminal.sizeX,         1,
      0,                        0,
      terminal::BufferType::TextBuffer
      (
        terminal::TextBuffer
        {
          flags:                terminal::TextBufferNull,
          chars:                String::from(""),
        }
      ),
    );
  /*let mut theEditor
  = terminal::TextBuffer::new
  (
    "",
    terminal::TextBufferNull
  );*/
  let     uidEditor = 0;
  /*
  = myTerminal.newBuffer
    (
      0,                        0,
      myTerminal.sizeX,         myTerminal.sizeY - 1,
      1,                        1,
      myTerminal.sizeX,         myTerminal.sizeY - 1,
      0,                        0,
      theEditor
    );
  */
  let     uidScreen
  = myTerminal.newBuffer
    (
      0,                        myTerminal.sizeY - 1,
      myTerminal.sizeX,         1,
      1,                        1,
      myTerminal.sizeX,         1,
      0,                        0,
      terminal::BufferType::ScreenBuffer
      (
        terminal::TerminalScreen
        {
          tilingMode:           terminal::TilingMode::AbsoluteResizeable,
          listOfBuffers:        vec!
          (
            uidStatusBar,
            uidEditor,
          ),
          rootOfBuffers:        uidStatusBar,
        }
      ),
    );
  
  myTerminal.addBufferToScreen
  (
    uidScreen,
    uidEditor
  );
  myTerminal.addBufferToScreen
  (
    uidScreen,
    uidStatusBar
  );
  
  myTerminal.newDisplay(uidScreen);

  'mainLoop:
    while let Some(ref events) = myTerminal.events
  {

    // handle stdin-events
    let event = events.try_recv();
    if event.is_ok()
    {
      match event.unwrap()
      {
        Event::Key(Key::Esc)                                =>
        {
          break 'mainLoop;
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
        _                                                   =>
        {
        }
      }
    }

    //
  }
}