use crate::
{
  Flags,
  display::
  {
    DisplayID,
  },
  frame::
  {
    FrameID,
  },
};

use std::
{
  sync::
  {
    mpsc::
    {
      channel,
      Receiver,
      Sender,
    },
  },
};

pub type EventReceiver                  =                                       Receiver<Event>;
pub type EventSender                    =                                       Sender<Event>;

bitflags!
{
  pub struct MouseButton: Flags
  {
    const None                          =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
    const LeftDown                      =                                       0b0000_0000_0000_0000_0000_0000_0000_0001;
    const MiddleDown                    =                                       0b0000_0000_0000_0000_0000_0000_0000_0010;
    const RightDown                     =                                       0b0000_0000_0000_0000_0000_0000_0000_0100;
  }
}

pub enum EventType
{
  Error(&'static str),
  Warning(&'static str),

  Character(char),
  Escape,
  Backspace,
  Return,
  Left,
  Right,
  Up,
  Down,
  Pause,
  Insert,
  Delete,
  Home,
  End,
  PageUp,
  PageDown,
  Function(u8),
  Alt(char),
  Ctrl(char),
  
  MouseOver,
  MouseLeftButtonPressed,
  MouseMiddleButtonPressed,
  MouseRightButtonPressed,
  MouseLeftButtonReleased,
  MouseMiddleButtonReleased,
  MouseRightButtonReleased,
  MouseWheelUp,
  MouseWheelDown,
  MouseMoveWithLeftButton,
  MouseMoveWithMiddleButton,
  MouseMoveWithRightButton,
  CursorPosition,
}

pub struct Event
{
  pub event:                            EventType,
  pub display:                          DisplayID,
  pub frame:                            FrameID,
  pub cursorX:                          usize,
  pub cursorY:                          usize,
  pub mouse:                            MouseButton,
}

impl Event
{
  pub fn new
  (
    event:                              EventType,
    display:                            DisplayID,
    frame:                              FrameID,
    cursorX:                            usize,
    cursorY:                            usize,
    mouse:                              MouseButton,
  ) -> Self
  {
    Self
    {
      event:                            event,
      display:                          display,
      frame:                            frame,
      cursorX:                          cursorX,
      cursorY:                          cursorY,
      mouse:                            mouse,
    }
  }
  pub fn openChannel
  (
  ) -> ( EventSender, EventReceiver )
  {
    channel()
  }
}
