require 't/commands/start'
require 'tempfile'
require 'time'

describe T::Commands::Start do
  subject(:command) { described_class.new(:out => stdout, :file => t_file, :time => time_stub) }
  let(:stdout) { StringIO.new }
  let(:tmpfile) { Tempfile.new('tspec') }
  let(:t_file) { tmpfile.path }
  let(:time_stub) { double('Time').tap { |x| x.stub('now') { @now } } }
  after { tmpfile.unlink }

  before { @now = Time.parse('2013-09-08 13:45:56') }

  context 'with no file' do
    before do
      File.unlink t_file
      command.run
    end
    it { expect(File.read(t_file)).to eq("2013-09-08 13:45,\n") }
    it { expect(stdout.string).to eq("Starting work.\n") }
  end

  context 'with an empty file' do
    before do
      command.run
    end
    it { expect(File.read(t_file)).to eq("2013-09-08 13:45,\n") }
    it { expect(stdout.string).to eq("Starting work.\n") }
  end

  context 'with some entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,2013-09-08 12:15
E_T
      command.run
    end
    it { expect(File.read(t_file)).to eq(<<E_T) }
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,2013-09-08 12:15
2013-09-08 13:45,
E_T
    it { expect(stdout.string).to eq("Starting work.\n") }
  end

  context 'with an incomplete entry in the file' do
    before do
      File.write(t_file, <<E_T)
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,
E_T
      command.run
    end
    it { expect(File.read(t_file)).to eq(<<E_T) }
2013-09-08 10:45,2013-09-08 11:45
2013-09-08 11:55,
E_T
    it { expect(stdout.string).to eq("You already started working, at 2013-09-08 11:55!\n") }
  end
end
