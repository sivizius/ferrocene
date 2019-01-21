pub mod style;

use crate::
{
  Flags,
  frame::
  {
    style::
    {
      Colour,
      StyledToken,
    },
  },
};

pub type FrameID                        =                                       usize;
type GridBorder                         =                                       usize;

bitflags!
{
  pub struct FrameFlag: Flags
  {
    const None                          =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
  }
}

pub struct StatusFrame
{
  pub flags:                            FrameFlag,
  pub offs:                             isize,
  pub text:                             String,
  pub bgChar:                           char,
}

pub struct TextFrame
{
  pub flags:                            FrameFlag,
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub lines:                            Vec<String>,
  pub bgChar:                           char,
}

pub struct EditorFrame
{
  pub flags:                            FrameFlag,
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub lines:                            Vec<Vec<StyledToken>>,
  pub bgChar:                           char,
}

pub enum PixelEncoding
{
  None,
  RGB   ( u8, u8, u8      ),
  RGBA  ( u8, u8, u8, u8  ),
  Sixel ( String          ),
}

pub struct PixelFrame
{
  pub offsX:                            isize,
  pub offsY:                            isize,
  pub sizeX:                            usize,
  pub sizeY:                            usize,
  pub scale:                            f64,
  pub ground:                           Colour,
  pub changed:                          bool,
  pub input:                            PixelEncoding,
  pub output:                           PixelEncoding,
}

pub struct PlotFrame
{
}

pub enum Tiling
{
  None,
  Grid,
}

pub struct Instance
{
  pub frame:                            FrameID,
  pub posX:                             isize,
  pub posY:                             isize,
  pub lenX:                             usize,
  pub lenY:                             usize,
  pub minX:                             usize,
  pub minY:                             usize,
  pub maxX:                             usize,
  pub maxY:                             usize,
  pub gridOriginX:                      GridBorder,
  pub gridOriginY:                      GridBorder,
  pub gridLenghtX:                      GridBorder,
  pub gridLenghtY:                      GridBorder,
}

pub struct ParentFrame
{
  pub typeOfTiling:                     Tiling,
  pub listOfInstances:                  Vec<Instance>,
  pub gridBordersX:                     Vec<isize>,
  pub gridBordersY:                     Vec<isize>,
  pub gridMinimumX:                     Vec<usize>,
  pub gridMinimumY:                     Vec<usize>,
  pub pivotFrame:                       FrameID,
}

pub struct LayerFrame
{
  pub listOfLayers:                     Vec<FrameID>,
}

pub enum Frame
{
  Status(StatusFrame),
  Text(TextFrame),
  Editor(EditorFrame),
  Pixel(PixelFrame),
  Plot(PlotFrame),
  Parent(ParentFrame),
  Layers(LayerFrame),
}

impl Frame
{
  pub fn newStatusFrame
  (
    flags:                              FrameFlag,
    offs:                               isize,
    text:                               String,
    bgChar:                             char,
  ) -> Frame
  {
    Frame::Status
    (
      StatusFrame
      {
        flags:                          flags,
        offs:                           offs,
        text:                           text,
        bgChar:                         bgChar,
      }
    )
  }

  pub fn newTextFrame
  (
    flags:                              FrameFlag,
    offsX:                              isize,
    offsY:                              isize,
    lines:                              Vec<String>,
    bgChar:                             char,
  ) -> Frame
  {
    Frame::Text
    (
      TextFrame
      {
        flags:                          flags,
        offsX:                          offsX,
        offsY:                          offsY,
        lines:                          lines,
        bgChar:                         bgChar,
      }
    )
  }

  pub fn newEditorFrame
  (
    flags:                              FrameFlag,
    offsX:                              isize,
    offsY:                              isize,
    lines:                              Vec<Vec<StyledToken>>,
    bgChar:                             char,
  ) -> Frame
  {
    Frame::Editor
    (
      EditorFrame
      {
        flags:                          flags,
        offsX:                          offsX,
        offsY:                          offsY,
        lines:                          lines,
        bgChar:                         bgChar,
      }
    )
  }

  pub fn newPixelFrame
  (
    offsX:                              isize,
    offsY:                              isize,
    sizeX:                              usize,
    sizeY:                              usize,
    scale:                              f64,
    ground:                             Colour,
    input:                              PixelEncoding,
    output:                             PixelEncoding,
  ) -> Frame
  {
    Frame::Pixel
    (
      PixelFrame
      {
        offsX:                          offsX,
        offsY:                          offsY,
        sizeX:                          sizeX,
        sizeY:                          sizeY,
        scale:                          scale,
        ground:                         ground,
        changed:                        true,
        input:                          input,
        output:                         output,
      }
    )
  }

  pub fn newInstance
  (
    frame:                              FrameID,
    posX:                               isize,
    posY:                               isize,
    lenX:                               usize,
    lenY:                               usize,
    minX:                               usize,
    minY:                               usize,
    maxX:                               usize,
    maxY:                               usize,
    gridOriginX:                        GridBorder,
    gridOriginY:                        GridBorder,
    gridLenghtX:                        GridBorder,
    gridLenghtY:                        GridBorder,
  ) -> Instance
  {
    Instance
    {
      frame:                            frame,
      posX:                             posX,
      posY:                             posY,
      lenX:                             lenX,
      lenY:                             lenY,
      minX:                             minX,
      minY:                             minY,
      maxX:                             maxX,
      maxY:                             maxY,
      gridOriginX:                      gridOriginX,
      gridOriginY:                      gridOriginY,
      gridLenghtX:                      gridLenghtX,
      gridLenghtY:                      gridLenghtY,
    }
  }

  pub fn newParentFrame
  (
    tiling:                             Tiling,
    listOfInstances:                    Vec<Instance>,
    mut gridBordersX:                   Vec<isize>,
    mut gridBordersY:                   Vec<isize>,
    mut gridMinimumX:                   Vec<usize>,
    mut gridMinimumY:                   Vec<usize>,
    pivotFrame:                         FrameID,
  ) -> Frame
  {
    gridBordersX.sort();
    gridBordersY.sort();
    gridMinimumX.resize( gridBordersX.len() - 1, 0 );
    gridMinimumY.resize( gridBordersY.len() - 1, 0 );
    Frame::Parent
    (
      ParentFrame
      {
        typeOfTiling:                   tiling,
        listOfInstances:                listOfInstances,
        gridBordersX:                   gridBordersX,
        gridBordersY:                   gridBordersY,
        gridMinimumX:                   gridMinimumX,
        gridMinimumY:                   gridMinimumY,
        pivotFrame:                     pivotFrame,
      }
    )
  }
}