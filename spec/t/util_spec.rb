require "spec_helper"

require "time"
require "t/util"

describe T::Util do
  context "add_days" do
    def pt(s)
      Time.parse(s)
    end

    # DST ends 2013-11-03 in the US
    it { expect(T::Util.add_days(pt("2013-11-01"), 1)).to eq(pt("2013-11-02")) }
    it { expect(T::Util.add_days(pt("2013-11-01"), 10)).to eq(pt("2013-11-11")) }
    it { expect(T::Util.add_days(pt("2013-11-02"), 10)).to eq(pt("2013-11-12")) }
    it { expect(T::Util.add_days(pt("2013-11-03"), 10)).to eq(pt("2013-11-13")) }

    # DST begins 2013-03-10 in the US
    it { expect(T::Util.add_days(pt("2013-03-04"), 5)).to eq(pt("2013-03-09")) }
    it { expect(T::Util.add_days(pt("2013-03-04"), 6)).to eq(pt("2013-03-10")) }
    it { expect(T::Util.add_days(pt("2013-03-04"), 7)).to eq(pt("2013-03-11")) }
  end
end
