use crate::yatui::*;
use crate::yatui::frame::*;
use crate::yatui::display::*;

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
    RwLock,
  },
  thread,
};

pub type EventReceiver                  =                                       Receiver<Event>;
pub type EventSender                    =                                       Sender<Event>;

pub const MouseButton_None:       Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
pub const MouseButton_LeftDown:   Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0001;
pub const MouseButton_MiddleDown: Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0010;
pub const MouseButton_RightDown:  Flags =                                       0b0000_0000_0000_0000_0000_0000_0000_0100;
pub enum EventType
{
  Error(&'static str),
  Char(char),
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
  pub mouse:                            Flags,
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
    mouse:                              Flags,
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
