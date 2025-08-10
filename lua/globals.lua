---@meta

PLAYER_1 = 0
PLAYER_2 = 1

---@param s string
---@return any
function JsonDecode(s)
end

---@param s any
---@return string
function JsonEncode(s)
end

---@param enum number
---@return string
function ToEnumShortString(enum)
end

---@class Def
---@field ActorFrame fun(t: ActorFrameTable): ActorFrameTable
Def = {}

---@class ActorFrameTable
---@field Class? string
---@field children? ActorFrameTable[]
---@field _Source? string
---@field _Line? number
---@field _Dir? string
---@field _Level? number
---@field [string] any


---@class ProfileManager
---@field GetPlayerName fun(self: ProfileManager, player: number): string
PROFILEMAN = {}


---@class NetworkManager
---@field WebSocket fun(self: NetworkManager, options: WebSocketOptions): WebSocket
NETWORK = {}

---@class WebSocketOptions
---@field url string
---@field headers? table<string, string>
---@field handshakeTimeout? number
---@field pingInterval? number
---@field automaticReconnect? boolean
---@field onMessage? fun(message: WebSocketMessage)

---@class WebSocketMessage
---@field type number
---@field data string

---@class WebSocket
---@field Close fun(self: WebSocket): nil
---@field Send fun(self: WebSocket, data: string, binary?: boolean): boolean
