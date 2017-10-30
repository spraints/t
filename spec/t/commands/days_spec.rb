require "spec_helper"
require "support/command_helpers"

require "t/commands/days"

describe T::Commands::Days do
  subject(:command) { described_class.new(:out => stdout, :file => t_file) }
  include CommandHelpers

  context "with no file" do
    before do
      File.unlink t_file
      command.run
    end
    it { expect(stdout.string).to eq("") }
  end

  context 'with an empty file' do
    before do
      command.run
    end
    it { expect(stdout.string).to eq("") }
  end

  context 'with some entries in the file' do
    before do
      File.write(t_file, <<E_T)
2013-08-01 10:45,2013-08-01 11:15
2013-08-02 10:15,2013-08-02 10:44
2013-08-11 10:45,2013-08-11 11:46
2013-08-22 10:45,2013-08-22 11:47
2013-08-31 10:45,2013-08-31 11:48
2013-09-04 10:45,2013-09-04 11:04
2013-09-04 11:04,2013-09-04 11:16
2013-09-04 11:16,2013-09-04 11:26
2013-09-04 11:16,2013-09-04 11:59
2013-09-05 11:26,2013-09-05 11:39
2013-09-05 11:39,2013-09-05 11:49
E_T
      command.run
    end

    let(:lines) { stdout.string.split("\n") }
    it { expect(lines[0]).to  eq "2013-07-28 - 2013-08-03 |     |     |     |     |  30 |  29 |     |    59" }
    it { expect(lines[1]).to  eq "2013-07                 |     |     |     |     |  30 |  29 |     |    59" }
    it { expect(lines[2]).to  eq "2013-08-04 - 2013-08-10 |     |     |     |     |     |     |     |     0" }
    it { expect(lines[3]).to  eq "2013-08-11 - 2013-08-17 |  61 |     |     |     |     |     |     |    61" }
    it { expect(lines[4]).to  eq "2013-08-18 - 2013-08-24 |     |     |     |     |  62 |     |     |    62" }
    it { expect(lines[5]).to  eq "2013-08-25 - 2013-08-31 |     |     |     |     |     |     |  63 |    63" }
    it { expect(lines[6]).to  eq "2013-08                 |  61 |     |     |     |  62 |     |  63 |   186" }
    it { expect(lines[7]).to  eq "2013-09-01 - 2013-09-07 |     |     |     |  84 |  23 |     |     |   107" }
    it { expect(lines[8]).to  eq "2013-09                 |     |     |     |  84 |  23 |     |     |   107" }
    it { expect(lines[9]).to  eq "2013                    |  61 |     |     |  84 | 115 |  29 |  63 |   352" }
    it { expect(lines[10]).to be_nil }
  end
end
