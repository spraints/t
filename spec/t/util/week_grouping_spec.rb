require "spec_helper"

require "time"
require "t/data"
require "t/util/week_grouping"

describe T::Util::WeekGrouping do
  subject { described_class.new(data) }
  let(:data) { T::Data.new(entries) }

  def vals
    res = []
    subject.each_week { |*week| res << week }
    res
  end

  def e(start, stop)
    T::Data::Entry.new(start, stop)
  end

  def t(s)
    Time.parse(s)
  end

  context "no entries" do
    let(:entries) { [] }

    it { expect(vals).to eq([]) }
  end

  context "mid-summer" do
    let(:entries) { [
      e("2013-07-01 07:00", "2013-07-01 08:00"),
      e("2013-07-08 07:00", "2013-07-08 08:00"),
      e("2013-07-10 07:00", "2013-07-10 08:00"),
      e("2013-07-15 07:00", "2013-07-15 08:00"),
    ] }

    it { expect(vals).to eq([
      [t("2013-06-30"), t("2013-07-07"), [entries[0]]],
      [t("2013-07-07"), t("2013-07-14"), [entries[1], entries[2]]],
      [t("2013-07-14"), t("2013-07-21"), [entries[3]]],
    ]) }
  end

  context "beginning of DST" do
    let(:entries) { [
      e("2013-03-01 07:00", "2013-03-01 08:00"),
      e("2013-03-08 07:00", "2013-03-08 08:00"),
      e("2013-03-11 07:00", "2013-03-11 08:00"),
      e("2013-03-15 07:00", "2013-03-15 08:00"),
      e("2013-03-22 07:00", "2013-03-22 08:00"),
      e("2013-03-29 07:00", "2013-03-29 08:00"),
    ] }

    it { expect(vals).to eq([
      [t("2013-02-24"), t("2013-03-03"), [entries[0]]],
      [t("2013-03-03"), t("2013-03-10"), [entries[1]]],
      [t("2013-03-10"), t("2013-03-17"), [entries[2], entries[3]]],
      [t("2013-03-17"), t("2013-03-24"), [entries[4]]],
      [t("2013-03-24"), t("2013-03-31"), [entries[5]]],
    ]) }
  end

  context "end of DST" do
    let(:entries) { [
      e("2013-11-01 07:00", "2013-11-01 08:00"),
      e("2013-11-08 07:00", "2013-11-08 08:00"),
      e("2013-11-11 07:00", "2013-11-11 08:00"),
      e("2013-11-15 07:00", "2013-11-15 08:00"),
      e("2013-11-22 07:00", "2013-11-22 08:00"),
      e("2013-11-29 07:00", "2013-11-29 08:00"),
    ] }

    it { expect(vals).to eq([
      [t("2013-10-27"), t("2013-11-03"), [entries[0]]],
      [t("2013-11-03"), t("2013-11-10"), [entries[1]]],
      [t("2013-11-10"), t("2013-11-17"), [entries[2], entries[3]]],
      [t("2013-11-17"), t("2013-11-24"), [entries[4]]],
      [t("2013-11-24"), t("2013-11-31"), [entries[5]]],
    ]) }
  end

  context "entry spans weeks" do
    let(:entries) { [
      e("2013-07-01 07:00", "2013-07-01 08:00"),
      e("2013-07-06 07:00", "2013-07-08 08:00"),
      e("2013-07-10 07:00", "2013-07-10 08:00"),
    ] }

    it { expect(vals).to eq([
      [t("2013-06-30"), t("2013-07-07"), [entries[0], entries[1]]],
      [t("2013-07-07"), t("2013-07-14"), [entries[1], entries[2]]],
    ]) }
  end
end
