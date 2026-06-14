local function fourcc(s)
    return string.byte(s, 1) * 0x1000000
        + string.byte(s, 2) * 0x10000
        + string.byte(s, 3) * 0x100
        + string.byte(s, 4)
end

function main()
    local xtr = CreateTrigger()
    TriggerRegisterPlayerEvent(xtr, Player(0), 17)
	TriggerAddAction(xtr, function()
	print("Main ran correctly")
    local x = 0.0
    local y = 0.0
    local p = Player(0)

    local keeper = CreateUnit(p, fourcc("Ekee"), x, y, 270.0)

    CBuffRejuvinationApply(
        fourcc("BEer"), -- buff RawEffect/int
        keeper,         -- unit
        999.0,          -- duration
        400.0,          -- healLife
        10.0,           -- healMana
        true,           -- allowFullLife
        true            -- allowFullMana
    )
	end)
end
