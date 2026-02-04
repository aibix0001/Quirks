-- Example Quirks configuration
-- Place this file at ~/.config/quirks/init.lua

-- Display a welcome message
quirks.print("Welcome to Quirks!")

-- Register custom keybindings
quirks.keymap("n", "<leader>w", ":w<CR>")      -- Save with leader+w
quirks.keymap("n", "<leader>q", ":q<CR>")      -- Quit with leader+q
quirks.keymap("n", "<leader>x", ":wq<CR>")     -- Save and quit

-- Set editor options (coming soon)
-- quirks.set_option("number", true)
-- quirks.set_option("relativenumber", true)
-- quirks.set_option("tabstop", 4)

-- Register custom commands
quirks.command("hello", "quirks.print('Hello from Lua!')")
quirks.command("time", "quirks.print(os.date('%Y-%m-%d %H:%M:%S'))")

-- API version check
if quirks.api_version >= 1 then
    quirks.print("Quirks " .. quirks.version .. " loaded")
end
