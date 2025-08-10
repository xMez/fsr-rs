---@type WebSocket?
local ws = nil

---@param s string
---@return nil
local function parseMessage(s)
	if not ws then
		return
	end

	local data = JsonDecode(s)

	if data.response_type == "active_player_broadcast" then
		local activePlayer = data.data.current_player
		local p1 = PROFILEMAN:GetPlayerName(PLAYER_1)
		local command = {}
		command["ChangePlayer"] = { name = p1 }
		if p1 ~= activePlayer then
			ws:Send(JsonEncode(command))
		end
	end
end

---@class ProfileSwitcher
---@field ScreenSelectMusic ActorFrameTable
local t = {}

t.ScreenSelectMusic = Def.ActorFrame {
	ModuleCommand = function(self)
		ws = NETWORK:WebSocket {
			url = "ws://localhost:3000/ws",
			automaticReconnect = true,
			onMessage = function(msg)
				local msgType = ToEnumShortString(msg.type)
				if msgType == "Message" then
					parseMessage(msg.data)
				end
			end,
		}
	end
}

return t
