# A sample Guardfile
# More info at https://github.com/guard/guard#readme

rspec_options = {
  cmd: "bin/rspec --fail-fast",
  all_after_pass: true,
  run_all: { cmd: "bin/rspec" },
}

guard :rspec, rspec_options do
  watch(%r{^spec/.+_spec\.rb$})
  watch(%r{^lib/(.+)\.rb$})     { |m| "spec/lib/#{m[1]}_spec.rb" }
  watch('spec/spec_helper.rb')  { "spec" }
end
