use std::rc::
{
  Rc,
  Weak
};
use core::cell::RefCell;

pub struct Elder<T>
{
  parent:     Option<Weak<RefCell<Elder<T>>>>,
  childs:     Vec<Rc<RefCell<Elder<T>>>>,
  pub mydata: Option<T>
}

impl<T> Elder<T>
{
  pub fn new
  (
  ) -> Self
  {
    Self
    {
      parent: None,
      childs: vec!(),
      mydata: None,
    }
  }
}