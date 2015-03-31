require "t/version"
require "t/activity_words"

module T
  DATA_FILE = ENV["T_DATA_FILE"] || File.join(ENV['HOME'], ".t.csv")

  DATE_FORMAT = "%Y-%m-%d"
  TIME_FORMAT = "%Y-%m-%d %H:%M"

  DEFAULT_SPARKS = %w(▁ ▂ ▃ ▄ ▅ ▆ ▇ )

  def self.activity_words
    @activity_words ||= T::ActivityWords.new
  end
end
