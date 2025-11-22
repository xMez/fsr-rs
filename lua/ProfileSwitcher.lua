---@type WebSocket?
local ws = nil

---@param data table
---@return nil
local function parseMessage(data)
	if not ws then
		return
	end

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
					local data = JsonDecode(msg.data)
					-- If we receive a message that is not active_player_broadcast, unsubscribe from that type
					-- We always unsubscribe (even if already done) in case the server was restarted
					if data.response_type and data.response_type ~= "active_player_broadcast" then
						local unsubscribeCommand = {}
						unsubscribeCommand["Unsubscribe"] = { event_types = { data.response_type } }
						if ws ~= nil then
							ws:Send(JsonEncode(unsubscribeCommand))
						end
					end
					parseMessage(data)
				end
			end,
		}
	end
}

return t
