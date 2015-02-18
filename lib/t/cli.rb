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
      when 'csv'
        require 't/commands/csv'
        T::Commands::CSV.new
      when 'pto-weeks'
        require 't/commands/pto_by_week'
        T::Commands::PtoByWeek.new
      when 'path'
        require 't'
        puts T::DATA_FILE
        exit 0
      when 'edit'
        require 't'
        system ENV['EDITOR'], T::DATA_FILE
        exit 0
      when nil
        puts "A command (start, stop, edit) or query (status, today, week, all, punchcard, path, pto-weeks) is required."
        exit 1
      else
        puts "Unsupported command: #{command_name}"
        exit 1
      end
    end
  end
end
