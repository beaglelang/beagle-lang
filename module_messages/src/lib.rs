use core::pos::BiPos;

#[derive(Debug, Clone)]
pub enum ModuleMessage{
    SourceRequest(BiPos),
    SourceResponse(String)
}