require 'spec_helper'
require 'support/command_helpers'

require 't/commands/short'

describe T::Commands::Short do
  subject(:command) { described_class.new(:out => stdout, :file => t_file, :step => 30) }
  include CommandHelpers

  context 'with no file' do
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
2013-09-05 11:26,2013-09-05 11:39
2013-09-05 11:39,2013-09-05 11:49
E_T
    end

    it { command.run; expect(stdout.string).to eq(<<SHORT) }
2013-07-28 - 2013-08-03     0/  2  (    0/   59 minutes)  ||
2013-08-04 - 2013-08-10     0/  0  (    0/    0 minutes)  
2013-08-11 - 2013-08-17     0/  1  (    0/   61 minutes)  |||
2013-08-18 - 2013-08-24     0/  1  (    0/   62 minutes)  |||
2013-08-25 - 2013-08-31     0/  1  (    0/   63 minutes)  |||
2013-09-01 - 2013-09-07     4/  5  (   45/   64 minutes)  ..|
SHORT

    it { command.run("60"); expect(stdout.string).to eq(<<SHORT) }
2013-07-28 - 2013-08-03     2/  2  (   59/   59 minutes)  ..
2013-08-04 - 2013-08-10     0/  0  (    0/    0 minutes)  
2013-08-11 - 2013-08-17     0/  1  (    0/   61 minutes)  |||
2013-08-18 - 2013-08-24     0/  1  (    0/   62 minutes)  |||
2013-08-25 - 2013-08-31     0/  1  (    0/   63 minutes)  |||
2013-09-01 - 2013-09-07     5/  5  (   64/   64 minutes)  ...
SHORT

  end
end
