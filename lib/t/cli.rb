module T
  class CLI
    def self.run!(argv)
      command = lookup(argv.shift)
      command.run(*argv)
      show_legend(command.legend_type)
    end

    def self.show_legend(legend_type)
      case legend_type
      when :week
        puts (1..5).to_a.map { |d| "#{d*8}h=#{d*8*60}m" }.join(' ')
      when :day
        puts "8h=#{8*60}m"
      end
    end

    def self.lookup(command_name)
      case command_name
      when 'start'
        require 't/commands/start'
        T::Commands::Start.new
      when 'stop'
        require 't/commands/stop'
        T::Commands::Stop.new
      when 'status'
        require 't/commands/status'
        T::Commands::Status.new
      when 'today'
        require 't/commands/today'
        T::Commands::Today.new
      when 'week'
        require 't/commands/week'
        T::Commands::Week.new
      when 'all'
        require 't/commands/all'
        T::Commands::All.new
      when 'punchcard'
        require 't/commands/punch_card'
        T::Commands::PunchCard.new
      when 'days'
        require 't/commands/days'
        T::Commands::Days.new
      when 'times'
        require 't/commands/times'
        T::Commands::Times.new
      when 'csv'
        require 't/commands/csv'
        T::Commands::CSV.new
      when 'svg'
        require 't/commands/svg'
        T::Commands::SVG.new
      when 'pto'
        require 't/commands/pto'
        T::Commands::Pto.new
      when 'short'
        require 't/commands/short'
        T::Commands::Short.new
      when 'path'
        require 't'
        puts T::DATA_FILE
        exit 0
      when 'edit'
        require 't'
        editor = ENV['EDITOR'] || 'vi'
        system 'sh', '-c', editor + ' "$@"', editor, T::DATA_FILE
        exit 0
      when nil
        puts "A command (start, stop, edit) or query (status, today, week, all, punchcard, days, csv, svg, pto, short, path) is required."
        exit 1
      else
        puts "Unsupported command: #{command_name}"
        exit 1
      end
    end
  end
end
