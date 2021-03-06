#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate bitflags;

pub mod display;
pub mod frame;
pub mod event;

pub use crate::
{
  display::
  {
    Display,
    DisplayFlag,
    DisplayID,
    DisplayType,
    ReadableFd,
    WriteableFd,
  },
  event::
  {
    Event,
    EventReceiver,
    EventSender,
    EventType,
    MouseButton,
  },
  frame::
  {
    EditorFrame,
    Frame,
    FrameID,
    FrameFlag,
    Instance,
    LayerFrame,
    ParentFrame,
    PixelEncoding,
    PixelFrame,
    PlotFrame,
    StatusFrame,
    TextFrame,
    Tiling,
    style::
    {
      Colour,
      StyledToken,
    },
  },
};

#[cfg(feature = "display-tty")]
use crate::display::tty::*;

use std::
{
  time::
  {
    SystemTime
  },
};

type Flags                              =                                       u32;

pub struct Ferrocene
{
  pub listOfDisplays:                   Vec<Option<Display>>,
  pub listOfFrames:                     Vec<Option<Frame>>,
  pub recvChannel:                      EventReceiver,
  pub sendChannel:                      EventSender,
}

impl Ferrocene
{
  pub fn new
  (
  ) -> Self
  {
    let ( sendChannel, recvChannel )    =                                       event::Event::openChannel();
    Self
    {
      listOfDisplays:                   vec!(),
      listOfFrames:                     vec!(),
      recvChannel:                      recvChannel,
      sendChannel:                      sendChannel,
    }
  }

  pub fn addDisplay
  (
    &mut self,
    mut display:                        Display,
  ) -> DisplayID
  {
    let id                              =                                       self.listOfDisplays.len() + 1;
    display.this                        =                                       id;
    self.listOfDisplays.push(Some(display));
    id
  }

  pub fn addFrame
  (
    &mut self,
    frame:                              Frame,
  ) -> FrameID
  {
    self.listOfFrames.push(Some(frame));
    self.listOfFrames.len()
  }

  #[cfg(feature = "display-tty")]
  pub fn addTTYDisplay
  (
    &mut self,
    flags:                              DisplayFlag,
    offsX:                              isize,
    offsY:                              isize,
    cursorX:                            usize,
    cursorY:                            usize,
    input:                              Box<display::ReadableFd>,
    output:                             Box<display::WriteableFd>,
    refreshRate:                        u64,
  ) -> ( usize, usize, DisplayID )
  {
    let display
    = TTYDisplay::new
      (
        flags,
        offsX,                          offsY,
        cursorX,                        cursorY,
        input,                          output,
        refreshRate,
      ).unwrap();
    (
      display.sizeX,
      display.sizeY,
      self.addDisplay(display)
    )
  }

  pub fn addStatusFrame
  (
    &mut self,
    flags:                              FrameFlag,
    offs:                               isize,
    text:                               String,
    bgChar:                             char,
  ) -> FrameID
  {
    self.addFrame
    (
      Frame::newStatusFrame
      (
        flags,
        offs,
        text,
        bgChar,
      )
    )
  }

  pub fn addTextFrame
  (
    &mut self,
    flags:                              FrameFlag,
    offsX:                              isize,
    offsY:                              isize,
    lines:                              Vec<String>,
    bgChar:                             char,
  ) -> FrameID
  {
    self.addFrame
    (
      Frame::newTextFrame
      (
        flags,
        offsX,                          offsY,
        lines,
        bgChar,
      )
    )
  }

  pub fn addEditorFrame
  (
    &mut self,
    flags:                              FrameFlag,
    offsX:                              isize,
    offsY:                              isize,
    lines:                              Vec<Vec<StyledToken>>,
    bgChar:                             char,
  ) -> FrameID
  {
    self.addFrame
    (
      Frame::newEditorFrame
      (
        flags,
        offsX,                          offsY,
        lines,
        bgChar,
      )
    )
  }
  
  pub fn addPixelFrame
  (
    &mut self,
    offsX:                              isize,
    offsY:                              isize,
    sizeX:                              usize,
    sizeY:                              usize,
    scale:                              f64,
    ground:                             Colour,
    input:                              PixelEncoding,
    output:                             PixelEncoding,
  ) -> FrameID
  {
    self.addFrame
    (
      Frame::newPixelFrame
      (
        offsX,                          offsY,
        sizeX,                          sizeY,
        scale,
        ground,
        input,                          output,
      )
    )
  }

  pub fn addParentFrame
  (
    &mut self,
    tiling:                             Tiling,
    listOfInstances:                    Vec<Instance>,
    gridBordersX:                       Vec<isize>,
    gridBordersY:                       Vec<isize>,
    gridMinimumX:                       Vec<usize>,
    gridMinimumY:                       Vec<usize>,
    pivotFrame:                         FrameID,
  ) -> FrameID
  {
    self.addFrame
    (
      Frame::newParentFrame
      (
        tiling,
        listOfInstances,
        gridBordersX,                   gridBordersY,
        gridMinimumX,                   gridMinimumY,
        pivotFrame,
      )
    )
  }

  pub fn accessFrame
  (
    &mut self,
    frame:                              FrameID,
  ) -> Result<&mut Frame, &str>
  {
    if frame == 0
    {
      Err("UID of frame cannot be zero")
    }
    else if frame > self.listOfFrames.len()
    {
      Err("UID of frame too high")
    }
    else if let Some(ref mut f) = self.listOfFrames [ frame - 1 ]
    {
      Ok(f)
    }
    else
    {
      Err("UID of frame invalidated.")
    }
  }

  pub fn accessDisplay
  (
    &mut self,
    display:                            DisplayID,
  ) -> Result<&mut Display, &str>
  {
    if display == 0
    {
      Err("UID of display cannot be zero")
    }
    else if display > self.listOfDisplays.len()
    {
      Err("UID of display too high")
    }
    else if let Some(ref mut d) = self.listOfDisplays [ display - 1 ]
    {
      Ok(d)
    }
    else
    {
      Err("UID of display invalidated.")
    }
  }

  pub fn setDisplayTitle
  (
    &mut self,
    display:                            DisplayID,
    title:                              String,
  )
  {
    let events                          =                                       self.sendChannel.clone();
    let refDisplay                      =                                       self.accessDisplay ( display ).unwrap();
    refDisplay.changeTitle( events, title );
  }

  pub fn turnOnDisplay
  (
    &mut self,
    display:                            DisplayID,
    frame:                              FrameID,
    title:                              String,
  ) -> Result<FrameID, &'static str>
  {
    let events                          =                                       self.sendChannel.clone();
    let refDisplay                      =                                       self.accessDisplay ( display ).unwrap();
    let mut fine: bool                  =                                       false;
    if let Ok(mut focusedFrame) = refDisplay.focusedFrame.lock()
    {
      refDisplay.flags                  |=                                      DisplayFlag::MaskRefresh;
      refDisplay.mainFrame              =                                       frame;
      **focusedFrame                    =                                       frame;
      fine                              =                                       true;
    }
    if fine
    {
      refDisplay.turnOn(events, title);
      Ok(refDisplay.mainFrame)
    }
    else
    {
      Err("")
    }
  }

  pub fn turnOffDisplay
  (
    &mut self,
    display:                            DisplayID,
  ) -> Result<FrameID, &'static str>
  {
    let events                          =                                       self.sendChannel.clone();
    let refDisplay                      =                                       self.accessDisplay ( display ).unwrap();
    let frame                           =                                       refDisplay.mainFrame;
    refDisplay.turnOff(events);
    Ok(frame)
  }

  pub fn render
  (
    &mut self,
  )
  {
    for display                         in                                      &mut self.listOfDisplays
    {
      let events                        =                                       self.sendChannel.clone();
      if let Some(ref mut refDisplay) = display
      {
        if  ( refDisplay.flags & DisplayFlag::MaskRefresh ) != DisplayFlag::None
        &&  ( refDisplay.lastRefresh.elapsed().unwrap() > refDisplay.nextRefresh )
        {
          refDisplay.flags              &=                                      !DisplayFlag::NeedRefresh;
          refDisplay.draw
          (
            &mut self.listOfFrames,
            &events,
            refDisplay.mainFrame,
            refDisplay.offsX,           refDisplay.offsY,
            refDisplay.sizeX,           refDisplay.sizeY,
          );
          #[cfg(any(feature = "display-tty"))]
          match &mut refDisplay.display
          {
            #[cfg(feature = "display-tty")]
            DisplayType::TTY(ref mut output)  =>                                output.flush( events, refDisplay.this ),
          }
          refDisplay.lastRefresh        =                                       SystemTime::now();
        }
      }
    }
  }
}