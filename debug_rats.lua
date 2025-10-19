-- Debug script for rats integration
print("=== Rats Debug Test ===")

-- Test 1: Check if rats binary exists and is executable
local rats_path = vim.fn.expand('~/.local/bin/rats')
print("1. Rats binary path: " .. rats_path)
print("   Exists: " .. tostring(vim.fn.filereadable(rats_path) == 1))
print("   Executable: " .. tostring(vim.fn.executable(rats_path) == 1))

-- Test 2: Test rats execution directly
print("\n2. Testing rats execution...")
local handle = io.popen('cd /tmp && ~/.local/bin/rats --help 2>&1 || echo "RATS_EXIT_CODE: $?"')
if handle then
  local result = handle:read("*a")
  handle:close()
  print("   Output: " .. (result or "nil"))
end

-- Test 3: Test with clean environment
print("\n3. Testing with clean environment...")
local clean_cmd = [[
cd /tmp && \
unset FZF_DEFAULT_COMMAND && \
unset FZF_DEFAULT_OPTS && \
export PATH="/Users/alexander/.local/bin:$PATH" && \
which rats && \
echo "RATS_FOUND" && \
timeout 1s rats 2>&1 || echo "RATS_TIMEOUT_EXPECTED"
]]

local handle2 = io.popen(clean_cmd)
if handle2 then
  local result2 = handle2:read("*a")
  handle2:close()
  print("   Clean env output: " .. (result2 or "nil"))
end

-- Test 4: Check current working directory
print("\n4. Current working directory: " .. vim.fn.getcwd())

-- Test 5: Test temporary file creation
local temp_file = vim.fn.tempname() .. '_test'
print("5. Temp file test: " .. temp_file)
local f = io.open(temp_file, 'w')
if f then
  f:write("test content")
  f:close()
  local read_f = io.open(temp_file, 'r')
  if read_f then
    local content = read_f:read("*a")
    read_f:close()
    print("   Temp file content: " .. content)
    os.remove(temp_file)
  end
end

print("\n=== Debug Complete ===")