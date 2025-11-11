#include "Callback.hpp"

CallbackHandler::CallbackHandler() {
  event_ = 0;
}

CallbackHandler::~CallbackHandler() {}

void CallbackHandler::add_callback(CallbackFunction pcb,
                                   uint32_t event,
                                   void *pud,
                                   CbInsertPolicy insert) {
  Callback cb{pcb, pud, event};
  if (insert == CB_ADD_END) {
    callbacks_.append(cb);
  } else {
    callbacks_.prepend(cb);
  }
}
