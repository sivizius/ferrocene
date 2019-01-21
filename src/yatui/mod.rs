pub mod display;
pub mod frame;
pub mod event;

pub use crate::yatui::display::*;
pub use crate::yatui::frame::*;
pub use crate::yatui::event::*;

use std::
{
  io::
  {
    self,
    Stdin,
    Stdout,
    Write,
  },
  mem,
  os::
  {
    unix::
    {
      io::
      {
        RawFd,
      },
    },
  },
  rc::
  {
    Rc,
    Weak,
  },
  str::
  {
  },
  thread,
  time::
  {
    Duration,
    SystemTime
  },
};

use unicode_segmentation::
{
  Graphemes,
  UnicodeSegmentation,
};

type Flags                              =                                       u32;

pub struct Yatui
{
  pub listOfDisplays:                   Vec<Option<Display>>,
  pub listOfFrames:                     Vec<Option<Frame>>,
  pub recvChannel:                      EventReceiver,
  pub sendChannel:                      EventSender,
}

impl Yatui
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

  pub fn addTTYDisplay
  (
    &mut self,
    flags:                              Flags,
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
    flags:                              Flags,
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
    flags:                              Flags,
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
    flags:                              Flags,
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

  pub fn turnOnDisplay
  (
    &mut self,
    display:                            DisplayID,
    frame:                              FrameID,
  ) -> Result<FrameID, &'static str>
  {
    let events                          =                                       self.sendChannel.clone();
    let refDisplay                      =                                       self.accessDisplay ( display ).unwrap();
    let mut fine: bool                  =                                       false;
    if let Ok(mut focusedFrame) = refDisplay.focusedFrame.lock()
    {
      refDisplay.flags                  |=                                      Display_MaskRefresh;
      refDisplay.mainFrame              =                                       frame;
      **focusedFrame                    =                                       frame;
      fine                              =                                       true;
    }
    if fine
    {
      refDisplay.turnOn(events);
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
        if  ( refDisplay.flags & Display_MaskRefresh ) != 0
        &&  ( refDisplay.lastRefresh.elapsed().unwrap() > refDisplay.nextRefresh )
        {
          refDisplay.flags              &=                                      !Display_NeedRefresh;
          refDisplay.draw
          (
            &mut self.listOfFrames,
            &events,
            refDisplay.mainFrame,
            refDisplay.offsX,           refDisplay.offsY,
            refDisplay.sizeX,           refDisplay.sizeY,
          );
          match &mut refDisplay.display
          {
            DisplayType::TTY(ref mut output) =>
            {
              let error                 =                                       output.output.flush();
              if !error.is_ok()
              {
                events.send
                (
                  event::Event::new
                  (
                    EventType::Error("cannot flush to tty"),
                    refDisplay.this,    0,
                    0,                  0,
                    0
                  )
                ).unwrap();
              }
            }
          }
          refDisplay.lastRefresh        =                                       SystemTime::now();
        }
      }
    }
  }
}