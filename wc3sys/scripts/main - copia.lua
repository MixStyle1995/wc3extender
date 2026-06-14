function main()
    local trig = CreateTrigger()
    TriggerRegisterPlayerEvent(trig, Player(0), 17)
	print("THIS IS A TOYOTA")
	print(ConvertPlayerEvent(17))
    
    local state = 0
    local myFrame = nil
    local myText = nil
    
    TriggerAddAction(trig, function()
        print("=== STATE MACHINE TEST === State: " .. state)
        local gameUI = BlzGetGameUI()
        if not gameUI or gameUI == 0 then return end
        BlzLoadTOCFile("UI\\FrameDef\\FrameDef.toc")
        if state == 0 then
            print("Welcome, hit ESC to start")
            TimerStart(CreateTimer(), 1.0, false, function()
                state = 1
                print("Timer fired, state is now 1. Hit ESC to create frame.")
            end)
        elseif state == 1 then
            myFrame = BlzCreateFrame("EscMenuBackdrop", gameUI, 0, 0)
            if myFrame and myFrame ~= 0 then
                BlzFrameClearAllPoints(myFrame)
                BlzFrameSetSize(myFrame, 0.3, 0.2)
                BlzFrameSetAbsPoint(myFrame, 4, 0.4, 0.5)
                BlzFrameSetTexture(myFrame, "ReplaceableTextures\\TeamColor\\TeamColor01.blp", 0, 1) -- Blue
                BlzFrameShow(myFrame, 1)
                
                myText = BlzCreateFrame("EscMenuMainPanelDialogTextTemplate", myFrame, 0, 0)
                if myText and myText ~= 0 then
                    BlzFrameClearAllPoints(myText)
                    BlzFrameSetAllPoints(myText, myFrame)
                    BlzTextFrameSetText(myText, "State 1: Blue Box")
                    BlzFrameShow(myText, 1)
                end
                print("Created Blue Box. Hit ESC for State 2.")
            end
        elseif state == 2 then
            if myFrame and myFrame ~= 0 then
                BlzFrameSetTexture(myFrame, "ReplaceableTextures\\CommandButtons\\BTNCorruptedEnt.blp", 0, 1)
                BlzFrameSetSize(myFrame, 0.4, 0.3)
                BlzFrameSetAbsPoint(myFrame, 4, 0.5, 0.6) -- Move and resize
                if myText and myText ~= 0 then
                    BlzTextFrameSetText(myText, "State 2: Ent & Resized")
                end
                print("Changed texture, size and pos. Hit ESC for State 3.")
            end
        elseif state == 3 then
            if myFrame and myFrame ~= 0 then
                -- 1074331748 is ControlClickEvent
                BlzFrameSetScript(myFrame, 1074069705 , function()
					print("we alive :D")
                    if myText and myText ~= 0 then
                        BlzTextFrameSetText(myText, "Clicked! Time: " .. os.clock())
                    end
                end)
                if myText and myText ~= 0 then
                    BlzTextFrameSetText(myText, "State 3: Click Me!")
                    BlzFrameSetTextColor(myText, 0xFFFF0000) -- Red text (ARGB)
                end
                print("Callback added. Click the box! Hit ESC for State 4.")
            end
        elseif state == 4 then
            if myText and myText ~= 0 then
                BlzTextFrameSetText(myText, "Destroying...")
            end
            TimerStart(CreateTimer(), 0.5, false, function()
                if myFrame and myFrame ~= 0 then
                    BlzDestroyFrame(myFrame)
                    myFrame = nil
                    myText = nil
                    print("Destroyed everything. State machine complete.")
                end
            end)
        end
        
        state = state + 1
    end)
end
