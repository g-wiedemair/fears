#pragma once

#include "fecore/FeParamList.hpp"
#include "fecore/fecore_api.hpp"

/// Base class for most classes in FeCore library and the base class for all
/// classes that can be registered with the framework
class FECORE_API FeCoreBase : public FeParamContainer {};
