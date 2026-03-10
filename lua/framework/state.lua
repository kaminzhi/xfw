local state = {}

local stores = {}

local Observable = {}
Observable.__index = Observable

function Observable:set(value)
  if self.value == value then
    return
  end
  self.value = value
  if self._emit then
    self._emit(self.id, value)
  end
end

function Observable:get()
  return self.value
end

local function create_observable(id, initial)
  local obs = setmetatable({ id = id, value = initial }, Observable)
  stores[id] = obs
  return obs
end

function state.observable(id, initial)
  return create_observable(id, initial)
end

function state.attach_emitter(cb)
  for _, store in pairs(stores) do
    store._emit = cb
  end
end

function state.interval(seconds, callback)
  table.insert(state._intervals or {}, { seconds = seconds, callback = callback })
end

return state
