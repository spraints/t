require 'tempfile'

module CommandHelpers
  def self.included(base)
    base.let(:stdout) { StringIO.new }
    base.let(:tmpfile) { Tempfile.new('tspec') }
    base.let(:t_file) { tmpfile.path }
    base.let(:time_stub) { double('Time').tap { |x| allow(x).to receive('now') { @now || Time.now } } }
    base.let(:tz_offset) { Time.now.strftime("%z") }
    base.after { tmpfile.unlink }
  end
end
