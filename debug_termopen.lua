-- Test termopen with rats
print("=== Testing termopen with rats ===")

local test_completed = false

-- Create a simple terminal test
local buf = vim.api.nvim_create_buf(false, true)
local temp_file = vim.fn.tempname() .. '_termopen_test'

print("Testing termopen execution...")
print("Temp file: " .. temp_file)

local job_id = vim.fn.termopen('/Users/alexander/.local/bin/rats > ' .. temp_file .. ' 2>&1 & echo $! > ' .. temp_file .. '.pid && sleep 0.5 && kill $(cat ' .. temp_file .. '.pid) 2>/dev/null || true', {
  on_exit = function(job_id, exit_code, event)
    print("Job exited with code: " .. exit_code)
    
    -- Read the output
    local file = io.open(temp_file, 'r')
    if file then
      local content = file:read('*all')
      file:close()
      print("Output length: " .. string.len(content))
      print("First 200 chars: " .. string.sub(content, 1, 200))
      os.remove(temp_file)
    else
      print("Could not read temp file")
    end
    
    -- Clean up pid file
    os.remove(temp_file .. '.pid')
    test_completed = true
  end,
  on_stdout = function(job_id, data, event)
    print("STDOUT: " .. vim.inspect(data))
  end,
  on_stderr = function(job_id, data, event)
    print("STDERR: " .. vim.inspect(data))
  end
})

print("Job ID: " .. (job_id or "nil"))

-- Wait a moment for the job to complete
local wait_count = 0
while not test_completed and wait_count < 50 do
  vim.loop.run('once')
  wait_count = wait_count + 1
end

if not test_completed then
  print("Test timed out")
end

print("=== Test Complete ===")