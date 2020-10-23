module T
  module Util
    # in seconds
    ONE_HOUR = 60 * 60
    ONE_DAY = ONE_HOUR * 24

    def self.add_days(date, n)
      (date + n * ONE_DAY + ONE_HOUR).to_date.to_time
    end
  end
end
