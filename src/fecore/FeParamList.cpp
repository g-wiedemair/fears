#include "FeParamList.hpp"

FeParamList::FeParamList(FeParamContainer *pc) : pc_(pc), current_group_(-1) {}

FeParamList::~FeParamList() {}

//-------------------------------------------------------------------------------------------------

FeParamContainer::FeParamContainer() : params_(nullptr) {}

FeParamContainer::~FeParamContainer() {}
