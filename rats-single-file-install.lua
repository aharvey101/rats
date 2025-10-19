-- Rats Fuzzy Finder - Complete installation in a single file
-- Place this file in ~/.config/nvim/lua/plugins/rats-fuzzy-finder.lua

return {
  {
    "rats-fuzzy-finder",
    dir = vim.fn.expand("~/.local/share/nvim/rats"), -- Local installation directory
    build = function()
      local install_dir = vim.fn.expand("~/.local/share/nvim/rats")
      local bin_dir = vim.fn.expand("~/.local/bin")
      
      -- Create directories
      vim.fn.mkdir(install_dir, "p")
      vim.fn.mkdir(bin_dir, "p")
      
      -- Download and compile rats
      local commands = {
        "cd " .. install_dir,
        "curl -L https://github.com/aharvey101/rats/archive/refs/heads/master.zip -o rats.zip",
        "unzip -o rats.zip",
        "cd rats-master",
        "cargo build --release",
        "cp target/release/rats " .. bin_dir .. "/",
        "cd ..",
        "rm -rf rats.zip rats-master"
      }
      
      local cmd = table.concat(commands, " && ")
      print("Installing rats fuzzy finder...")
      local result = vim.fn.system(cmd)
      
      if vim.v.shell_error == 0 then
        print("✅ Rats fuzzy finder installed successfully!")
      else
        print("❌ Installation failed: " .. result)
      end
    end,
    config = function()
      -- Fuzzy finder implementation
      local M = {}
      
      function M.find_files()
        local initial_cwd = vim.fn.getcwd()
        
        -- Create floating window
        local buf = vim.api.nvim_create_buf(false, true)
        local width = math.floor(vim.o.columns * 0.8)
        local height = math.floor(vim.o.lines * 0.8)
        
        local win = vim.api.nvim_open_win(buf, true, {
          relative = 'editor',
          width = width,
          height = height,
          col = math.floor((vim.o.columns - width) / 2),
          row = math.floor((vim.o.lines - height) / 2),
          style = 'minimal',
          border = 'rounded',
          title = ' 🦀 Rats Fuzzy Finder ',
          title_pos = 'center'
        })
        
        -- State for the picker
        local state = {
          query = '',
          results = {},
          selected_idx = 1,
          display_start = 1,
          current_dir = initial_cwd
        }
        
        -- Function to update results based on query
        local function update_results()
          local cmd = string.format('cd "%s" && "%s" --json --query "%s"', 
            state.current_dir, 
            vim.fn.expand('~/.local/bin/rats'), 
            state.query)
          
          local handle = io.popen(cmd)
          if handle then
            local output = handle:read('*all')
            handle:close()
            
            -- Parse JSON results
            local ok, results = pcall(vim.fn.json_decode, output)
            if ok and type(results) == 'table' then
              state.results = results
            else
              state.results = {}
            end
            
            -- Reset selection
            state.selected_idx = 1
            state.display_start = 1
          else
            state.results = {}
          end
        end
        
        -- Function to render the picker
        local function render()
          local lines = {}
          local highlights = {}
          
          -- Show current directory and prompt line
          local dir_indicator = string.format("📁 %s", vim.fn.fnamemodify(state.current_dir, ':~'))
          table.insert(lines, dir_indicator)
          table.insert(lines, '> ' .. state.query)
          
          -- Add separator
          table.insert(lines, string.rep('─', width - 4))
          
          -- Calculate display range
          local display_height = height - 7
          local display_end = math.min(state.display_start + display_height - 1, #state.results)
          
          -- Adjust display_start if needed
          if state.selected_idx < state.display_start then
            state.display_start = state.selected_idx
          elseif state.selected_idx >= state.display_start + display_height then
            state.display_start = state.selected_idx - display_height + 1
          end
          
          -- Recalculate display_end
          display_end = math.min(state.display_start + display_height - 1, #state.results)
          
          -- Display results
          for i = state.display_start, display_end do
            local result = state.results[i]
            if result then
              local icon = result.is_dir and '📁' or '📄'
              local line = string.format('  %s %s', icon, result.name)
              
              -- Highlight selected line
              if i == state.selected_idx then
                line = '> ' .. line:sub(3)
                table.insert(highlights, {line_nr = #lines + 1, hl_group = 'PmenuSel'})
              end
              
              table.insert(lines, line)
            end
          end
          
          -- Update buffer content
          vim.api.nvim_buf_set_option(buf, 'modifiable', true)
          vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
          vim.api.nvim_buf_set_option(buf, 'modifiable', false)
          
          -- Apply highlights
          for _, hl in ipairs(highlights) do
            vim.api.nvim_buf_add_highlight(buf, -1, hl.hl_group, hl.line_nr - 1, 0, -1)
          end
          
          -- Set cursor position
          vim.api.nvim_win_set_cursor(win, {2, #state.query + 2})
        end
        
        -- Function to navigate into a directory
        local function enter_directory(dir_path)
          state.current_dir = dir_path
          state.query = ''
          state.selected_idx = 1
          state.display_start = 1
          update_results()
          render()
        end
        
        -- Function to go back to parent directory
        local function go_back()
          local parent = vim.fn.fnamemodify(state.current_dir, ':h')
          if parent ~= state.current_dir then
            enter_directory(parent)
          end
        end
        
        -- Set up buffer options
        vim.api.nvim_buf_set_option(buf, 'buftype', 'nofile')
        vim.api.nvim_buf_set_option(buf, 'swapfile', false)
        vim.api.nvim_buf_set_option(buf, 'bufhidden', 'wipe')
        
        -- Set up keymaps
        local opts = { buffer = buf, silent = true }
        
        -- Navigation
        vim.keymap.set('n', 'j', function()
          if state.selected_idx < #state.results then
            state.selected_idx = state.selected_idx + 1
            render()
          end
        end, opts)
        
        vim.keymap.set('n', 'k', function()
          if state.selected_idx > 1 then
            state.selected_idx = state.selected_idx - 1
            render()
          end
        end, opts)
        
        -- Enter insert mode
        vim.keymap.set('n', 'i', function() vim.cmd('startinsert') end, opts)
        
        -- Selection
        vim.keymap.set('n', '<CR>', function()
          if state.results[state.selected_idx] then
            local selected_item = state.results[state.selected_idx]
            if selected_item.is_dir then
              enter_directory(selected_item.path)
            else
              vim.api.nvim_win_close(win, true)
              vim.cmd('edit ' .. vim.fn.fnameescape(selected_item.path))
            end
          end
        end, opts)
        
        -- Cancel
        vim.keymap.set('n', '<Esc>', function() vim.api.nvim_win_close(win, true) end, opts)
        vim.keymap.set('n', 'q', function() vim.api.nvim_win_close(win, true) end, opts)
        
        -- Insert mode mappings
        vim.keymap.set('i', '<Esc>', function() vim.cmd('stopinsert') end, opts)
        vim.keymap.set('i', '<CR>', function()
          if state.results[state.selected_idx] then
            local selected_item = state.results[state.selected_idx]
            if selected_item.is_dir then
              enter_directory(selected_item.path)
            else
              vim.api.nvim_win_close(win, true)
              vim.cmd('edit ' .. vim.fn.fnameescape(selected_item.path))
            end
          end
        end, opts)
        
        -- Character input
        vim.api.nvim_create_autocmd('InsertCharPre', {
          buffer = buf,
          callback = function()
            local char = vim.v.char
            if char:match('[%w%s%p]') then
              state.query = state.query .. char
              update_results()
              vim.schedule(render)
            end
            vim.v.char = ''
          end
        })
        
        vim.keymap.set('i', '<BS>', function()
          if #state.query > 0 then
            state.query = state.query:sub(1, -2)
            update_results()
            render()
          end
        end, opts)
        
        -- Initial load
        update_results()
        render()
      end
      
      -- Set up keymaps
      vim.keymap.set('n', '<leader>ff', M.find_files, { desc = 'Find files (rats fuzzy finder)' })
      vim.keymap.set('n', '<leader>rf', M.find_files, { desc = 'Rats find files' })
      vim.keymap.set('n', '<C-p>', M.find_files, { desc = 'Find files (Ctrl+P style)' })
    end,
    keys = {
      { "<leader>ff", desc = "Find files (rats fuzzy finder)" },
      { "<leader>rf", desc = "Rats find files" },
      { "<C-p>", desc = "Find files (Ctrl+P style)" },
    },
  }
}