local M = {}

local config = require("stynx.config")

function M.run(prompt, on_output, on_done, on_error)
  local cmd = { config.options.binary, "--json" }

  if config.options.max_turns then
    table.insert(cmd, "--max-turns")
    table.insert(cmd, tostring(config.options.max_turns))
  end

  if config.options.model then
    table.insert(cmd, "--model")
    table.insert(cmd, config.options.model)
  end

  if config.options.effort then
    table.insert(cmd, "--effort")
    table.insert(cmd, config.options.effort)
  end

  table.insert(cmd, "-p")
  table.insert(cmd, prompt)

  local stdout = {}
  local stderr = {}

  local job_id = vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(stdout, line)
          end
        end
      end
      if on_output then
        on_output(data)
      end
    end,
    on_stderr = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(stderr, line)
          end
        end
      end
    end,
    on_exit = function(_, code)
      if code == 0 then
        if on_done then
          local raw = table.concat(stdout, "\n")
          on_done(raw)
        end
      else
        local err = table.concat(stderr, "\n")
        if on_error then
          on_error("stynx exited with code " .. tostring(code) .. ": " .. err)
        end
      end
    end,
  })

  return job_id
end

function M.run_stream(prompt, on_chunk, on_done, on_error)
  local cmd = { config.options.binary }

  if config.options.max_turns then
    table.insert(cmd, "--max-turns")
    table.insert(cmd, tostring(config.options.max_turns))
  end

  if config.options.model then
    table.insert(cmd, "--model")
    table.insert(cmd, config.options.model)
  end

  table.insert(cmd, "-p")
  table.insert(cmd, prompt)

  local full_output = {}
  local stderr = {}

  local job_id = vim.fn.jobstart(cmd, {
    on_stdout = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(full_output, line)
            if on_chunk then
              on_chunk(line)
            end
          end
        end
      end
    end,
    on_stderr = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(stderr, line)
          end
        end
      end
    end,
    on_exit = function(_, code)
      if code == 0 then
        if on_done then
          on_done(table.concat(full_output, "\n"))
        end
      else
        local err = table.concat(stderr, "\n")
        if on_error then
          on_error("stynx exited with code " .. tostring(code) .. ": " .. err)
        end
      end
    end,
  })

  return job_id
end

return M
